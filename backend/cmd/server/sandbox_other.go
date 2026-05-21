//go:build !linux

package main

import "os/exec"

// setupSandbox is a no-op on non-Linux platforms.
func setupSandbox(gitBinary, reposRoot, staticRoot string) {}

// hardenCmd is a no-op on non-Linux platforms.
func hardenCmd(cmd *exec.Cmd) {}
