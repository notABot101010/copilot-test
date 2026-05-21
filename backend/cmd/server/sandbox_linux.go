//go:build linux

package main

import (
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"syscall"
	"unsafe"

	"golang.org/x/sys/unix"
)

// setupSandbox applies Landlock filesystem sandboxing to the current process so
// that both the server and every git subprocess it spawns can only access the
// paths they legitimately need. It also sets PR_SET_NO_NEW_PRIVS on the whole
// process tree, preventing privilege escalation via execve.
//
// The function is best-effort: if the running kernel does not support Landlock
// (Linux < 5.13) the call succeeds silently and only logs a notice.
func setupSandbox(gitBinary, reposRoot, staticRoot string) {
	if err := landlockRestrict(gitBinary, reposRoot, staticRoot); err != nil {
		log.Printf("sandbox: landlock not applied (%v); continuing without filesystem sandboxing", err)
		return
	}
	log.Printf("sandbox: landlock filesystem sandboxing active")
}

// landlockRestrict builds and applies a Landlock ruleset to the calling
// process. Allowed paths:
//
//   - Read + execute: git binary directory, /usr, /bin, /lib*, /etc, /proc, /dev
//   - Read-only: staticRoot (frontend assets, if it exists)
//   - Read + write: reposRoot, os.TempDir()
func landlockRestrict(gitBinary, reposRoot, staticRoot string) error {
	// Probe the kernel's Landlock ABI version. Returns ENOSYS if the syscall
	// doesn't exist, or EOPNOTSUPP if Landlock is compiled out.
	abiRaw, _, errno := unix.Syscall(unix.SYS_LANDLOCK_CREATE_RULESET,
		0, 0, unix.LANDLOCK_CREATE_RULESET_VERSION)
	if errno != 0 {
		if errors.Is(errno, unix.ENOSYS) || errors.Is(errno, unix.EOPNOTSUPP) {
			return fmt.Errorf("not supported by kernel (errno %d)", errno)
		}
		return fmt.Errorf("abi version probe: %w", errno)
	}
	_ = int(abiRaw) // abiVersion; V1 rights are sufficient for our use-case

	// V1 Landlock access rights (Linux 5.13+). We only claim rights we
	// actually need; the kernel rejects unknown bits, so never use V2/V3/V4
	// rights on a V1 kernel.
	const v1Rights = uint64(
		unix.LANDLOCK_ACCESS_FS_EXECUTE |
			unix.LANDLOCK_ACCESS_FS_WRITE_FILE |
			unix.LANDLOCK_ACCESS_FS_READ_FILE |
			unix.LANDLOCK_ACCESS_FS_READ_DIR |
			unix.LANDLOCK_ACCESS_FS_REMOVE_DIR |
			unix.LANDLOCK_ACCESS_FS_REMOVE_FILE |
			unix.LANDLOCK_ACCESS_FS_MAKE_CHAR |
			unix.LANDLOCK_ACCESS_FS_MAKE_DIR |
			unix.LANDLOCK_ACCESS_FS_MAKE_REG |
			unix.LANDLOCK_ACCESS_FS_MAKE_SOCK |
			unix.LANDLOCK_ACCESS_FS_MAKE_FIFO |
			unix.LANDLOCK_ACCESS_FS_MAKE_BLOCK |
			unix.LANDLOCK_ACCESS_FS_MAKE_SYM,
	)

	attr := unix.LandlockRulesetAttr{Access_fs: v1Rights}
	rulesetFdRaw, _, errno := unix.Syscall(unix.SYS_LANDLOCK_CREATE_RULESET,
		uintptr(unsafe.Pointer(&attr)), unsafe.Sizeof(attr), 0)
	if errno != 0 {
		return fmt.Errorf("landlock_create_ruleset: %w", errno)
	}
	rulesetFd := int(rulesetFdRaw)
	defer unix.Close(rulesetFd)

	// addRule opens path with O_PATH (no actual file access), creates a
	// Landlock rule, then closes the fd. Errors are logged but non-fatal so
	// that missing optional paths (e.g. /lib64 on some distros) don't abort.
	addRule := func(path string, access uint64) {
		if _, err := os.Stat(path); err != nil {
			return // path doesn't exist on this system; skip
		}
		fd, err := unix.Open(path, unix.O_PATH|unix.O_CLOEXEC, 0)
		if err != nil {
			log.Printf("sandbox: landlock open %s: %v", path, err)
			return
		}
		defer unix.Close(fd)
		rule := unix.LandlockPathBeneathAttr{
			Allowed_access: access,
			Parent_fd:      int32(fd),
		}
		const landlockRulePathBeneath = 1
		_, _, errno := unix.Syscall6(unix.SYS_LANDLOCK_ADD_RULE,
			uintptr(rulesetFd), landlockRulePathBeneath,
			uintptr(unsafe.Pointer(&rule)), 0, 0, 0)
		if errno != 0 {
			log.Printf("sandbox: landlock add rule for %s: %v", path, errno)
		}
	}

	const readExec = uint64(
		unix.LANDLOCK_ACCESS_FS_EXECUTE |
			unix.LANDLOCK_ACCESS_FS_READ_FILE |
			unix.LANDLOCK_ACCESS_FS_READ_DIR,
	)
	const readOnly = uint64(
		unix.LANDLOCK_ACCESS_FS_READ_FILE |
			unix.LANDLOCK_ACCESS_FS_READ_DIR,
	)
	const readWrite = uint64(
		unix.LANDLOCK_ACCESS_FS_READ_FILE |
			unix.LANDLOCK_ACCESS_FS_READ_DIR |
			unix.LANDLOCK_ACCESS_FS_WRITE_FILE |
			unix.LANDLOCK_ACCESS_FS_REMOVE_DIR |
			unix.LANDLOCK_ACCESS_FS_REMOVE_FILE |
			unix.LANDLOCK_ACCESS_FS_MAKE_DIR |
			unix.LANDLOCK_ACCESS_FS_MAKE_REG |
			unix.LANDLOCK_ACCESS_FS_MAKE_SYM,
	)

	// Executable system paths (git binary location, system libraries).
	for _, p := range []string{
		filepath.Dir(gitBinary), // e.g. /usr/bin
		"/usr",
		"/bin",
		"/lib",
		"/lib64",
		"/lib32",
		"/libx32",
	} {
		addRule(p, readExec)
	}

	// Read-only system paths (DNS resolution, TLS certificates, /proc, devices).
	for _, p := range []string{"/etc", "/proc", "/dev", "/sys"} {
		addRule(p, readOnly)
	}

	// Frontend static files (read-only; may not exist during development).
	if staticRoot != "" {
		addRule(staticRoot, readOnly)
	}

	// Read-write: repositories and temporary working trees.
	for _, p := range []string{reposRoot, os.TempDir()} {
		if err := os.MkdirAll(p, 0o755); err == nil {
			addRule(p, readWrite)
		}
	}

	// Prevent privilege escalation for this process and all future children.
	// This must be called before landlock_restrict_self.
	if err := unix.Prctl(unix.PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0); err != nil {
		return fmt.Errorf("prctl PR_SET_NO_NEW_PRIVS: %w", err)
	}

	// Apply the ruleset to the current process (inherited by all children).
	_, _, errno = unix.Syscall(unix.SYS_LANDLOCK_RESTRICT_SELF,
		uintptr(rulesetFd), 0, 0)
	if errno != 0 {
		return fmt.Errorf("landlock_restrict_self: %w", errno)
	}

	return nil
}

// hardenCmd applies OS-level process isolation to a git subprocess before it
// starts. It ensures the child is killed when the server dies and cannot gain
// new privileges via execve.
func hardenCmd(cmd *exec.Cmd) {
	if cmd.SysProcAttr == nil {
		cmd.SysProcAttr = &syscall.SysProcAttr{}
	}
	// SIGKILL the subprocess if the server process dies (prevents orphaned
	// git processes that keep file locks open).
	cmd.SysProcAttr.Pdeathsig = syscall.SIGKILL
}
