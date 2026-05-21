import {
  type DragEndEvent,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { CSS } from '@dnd-kit/utilities'
import {
  ActionIcon,
  AppShell,
  Autocomplete,
  Badge,
  Button,
  Card,
  Container,
  Group,
  Select,
  Stack,
  Text,
  TextInput,
  Textarea,
  Title,
} from '@mantine/core'
import { notifications } from '@mantine/notifications'
import { useSignals } from '@preact/signals-react/runtime'
import { type ReactNode, useCallback, useEffect, useMemo, useState } from 'react'
import { Link, Navigate, Route, Routes, useNavigate, useParams } from 'react-router-dom'
import {
  addIssueComment,
  addMergeRequestComment,
  addOrganizationMember,
  addSSHKey,
  createIssue,
  createMergeRequest,
  createOrganization,
  createProject,
  createRepoBranch,
  createRepoTag,
  createUser,
  deleteRepoBranch,
  deleteRepoFile,
  getMergeRequest,
  getMergeRequestDiff,
  getMergeRequestMergeStatus,
  getProjectSettings,
  getRepoBlame,
  getRepoCommit,
  getRepoFile,
  listIssues,
  listMergeRequests,
  listOrganizationMembers,
  listOrganizations,
  listProjects,
  listRepoBranches,
  listRepoCommits,
  listRepoTags,
  listRepoTree,
  listUsers,
  mergeMergeRequest,
  removeOrganizationMember,
  saveRepoFile,
  updateIssue,
  updateOrganizationMember,
  updateProjectSettings,
  type Issue,
  type MergeRequest,
  type Organization,
  type OrganizationMember,
  type Project,
  type RepoBlameLine,
  type RepoBranch,
  type RepoCommit,
  type RepoCommitDetails,
  type RepoEntry,
  type RepoTag,
  type User,
} from './api'
import { sessionSignal } from './state'

const ROLE_OPTIONS: { value: OrganizationMember['role']; label: string }[] = [
  { value: 'owner', label: 'Owner' },
  { value: 'admin', label: 'Admin' },
  { value: 'developer', label: 'Developer' },
  { value: 'viewer', label: 'Viewer' },
]

function App() {
  return (
    <AppShell padding="md">
      <AppShell.Main>
        <Container size="lg">
          <Stack>
            <Group justify="space-between">
              <Title order={2}>Go Git Hosting Platform</Title>
              <AuthPill />
            </Group>
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/projects/:projectId/repo" element={<RepositoryBrowser />} />
              <Route path="/projects/:projectId/issues" element={<IssueBoard />} />
              <Route path="/projects/:projectId/merge-requests" element={<MergeRequestBoard />} />
              <Route path="/projects/:projectId/merge-requests/:mergeRequestId" element={<MergeRequestDetails />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  )
}

function AuthPill() {
  useSignals()
  const session = sessionSignal.value
  return session ? <Badge variant="light">{session.username}</Badge> : <Badge color="gray">Guest</Badge>
}

function Home() {
  useSignals()
  const session = sessionSignal.value
  return (
    <Stack>
      {!session && <Onboard />}
      {session && (
        <>
          <SSHKeyPanel />
          <OrganizationsPanel />
          <ProjectsPanel />
        </>
      )}
    </Stack>
  )
}

function Onboard() {
  const [username, setUsername] = useState('')
  const save = async () => {
    if (!username.trim()) return
    const session = await createUser(username.trim())
    notifications.show({
      title: 'User created',
      message: `Token: ${session.token}. Use this for HTTP push auth.`,
    })
  }
  return (
    <Card withBorder>
      <Stack>
        <Title order={4}>Create user</Title>
        <TextInput value={username} onChange={(e) => setUsername(e.currentTarget.value)} />
        <Button onClick={save}>Create account</Button>
      </Stack>
    </Card>
  )
}

function SSHKeyPanel() {
  useSignals()
  const session = sessionSignal.value
  const [key, setKey] = useState('')
  const submit = async () => {
    if (!session) return
    await addSSHKey(session.userId, key)
    setKey('')
    notifications.show({ title: 'SSH key added', message: 'You can now push with SSH.' })
  }
  return (
    <Card withBorder>
      <Stack>
        <Title order={4}>SSH keys</Title>
        <Textarea value={key} minRows={3} onChange={(e) => setKey(e.currentTarget.value)} placeholder="ssh-ed25519 AAAA..." />
        <Button onClick={submit}>Add SSH key</Button>
      </Stack>
    </Card>
  )
}

function OrganizationsPanel() {
  useSignals()
  const session = sessionSignal.value
  const [name, setName] = useState('')
  const [orgs, setOrgs] = useState<Organization[]>([])
  const [users, setUsers] = useState<User[]>([])
  const [selectedOrgId, setSelectedOrgId] = useState<string | null>(null)
  const [members, setMembers] = useState<OrganizationMember[]>([])
  const [memberUserId, setMemberUserId] = useState<string | null>(null)
  const [memberRole, setMemberRole] = useState<OrganizationMember['role']>('developer')

  const load = useCallback(async () => {
    const [orgData, userData] = await Promise.all([listOrganizations(), listUsers()])
    setOrgs(orgData)
    setUsers(userData)
    const nextOrgId = selectedOrgId ?? (orgData[0] ? String(orgData[0].id) : null)
    setSelectedOrgId(nextOrgId)
    if (nextOrgId) {
      setMembers(await listOrganizationMembers(Number(nextOrgId)))
    } else {
      setMembers([])
    }
  }, [selectedOrgId])

  useEffect(() => {
    queueMicrotask(() => {
      load().catch(() => undefined)
    })
  }, [load])

  const create = async () => {
    await createOrganization(name)
    setName('')
    await load()
  }

  const addMember = async () => {
    if (!selectedOrgId || !memberUserId) return
    await addOrganizationMember(Number(selectedOrgId), Number(memberUserId), memberRole)
    await load()
  }

  const changeRole = async (userId: number, role: OrganizationMember['role']) => {
    if (!selectedOrgId) return
    await updateOrganizationMember(Number(selectedOrgId), userId, role)
    await load()
  }

  const removeMember = async (userId: number) => {
    if (!selectedOrgId) return
    await removeOrganizationMember(Number(selectedOrgId), userId)
    await load()
  }

  const selectedOrg = orgs.find((org) => String(org.id) === selectedOrgId)
  const memberOptions = users.map((user) => ({ value: String(user.id), label: `#${user.id} ${user.username}` }))

  return (
    <Card withBorder>
      <Stack>
        <Title order={4}>Organizations</Title>
        <Group align="end">
          <TextInput value={name} onChange={(e) => setName(e.currentTarget.value)} placeholder="acme" label="New organization" />
          <Button onClick={create}>Create</Button>
        </Group>
        <Select
          label="Manage organization"
          data={orgs.map((org) => ({ value: String(org.id), label: `#${org.id} ${org.name}` }))}
          value={selectedOrgId}
          onChange={setSelectedOrgId}
        />
        {orgs.map((org) => (
          <Text key={org.id}>
            #{org.id} {org.name} · owner #{org.ownerId}
          </Text>
        ))}
        {selectedOrg && (
          <Card withBorder>
            <Stack>
              <Title order={5}>Members for {selectedOrg.name}</Title>
              <Group align="end">
                <Select label="User" data={memberOptions} value={memberUserId} onChange={setMemberUserId} searchable />
                <Select
                  label="Role"
                  data={ROLE_OPTIONS}
                  value={memberRole}
                  onChange={(value) => setMemberRole((value as OrganizationMember['role']) || 'developer')}
                />
                <Button onClick={addMember}>Add member</Button>
              </Group>
              {members.length === 0 && <Text size="sm">No members yet.</Text>}
              {members.map((member) => (
                <Card key={member.userId} withBorder>
                  <Group justify="space-between" align="end">
                    <Text>
                      User #{member.userId} · {member.role}
                    </Text>
                    <Group align="end">
                      <Select
                        data={ROLE_OPTIONS}
                        value={member.role}
                        onChange={(value) => value && changeRole(member.userId, value as OrganizationMember['role'])}
                        disabled={member.userId === selectedOrg.ownerId || !session}
                      />
                      <Button
                        size="xs"
                        color="red"
                        variant="light"
                        onClick={() => removeMember(member.userId)}
                        disabled={member.userId === selectedOrg.ownerId || !session}
                      >
                        Remove
                      </Button>
                    </Group>
                  </Group>
                </Card>
              ))}
            </Stack>
          </Card>
        )}
      </Stack>
    </Card>
  )
}

function ProjectsPanel() {
  const [projects, setProjects] = useState<Project[]>([])
  const [orgs, setOrgs] = useState<Organization[]>([])
  const [name, setName] = useState('')
  const [orgID, setOrgID] = useState<string | null>(null)
  const navigate = useNavigate()

  const load = useCallback(async () => {
    const [projectData, orgData] = await Promise.all([listProjects(), listOrganizations()])
    setProjects(projectData)
    setOrgs(orgData)
    if (!orgID && orgData.length > 0) setOrgID(String(orgData[0].id))
  }, [orgID])

  useEffect(() => {
    queueMicrotask(() => {
      load().catch(() => undefined)
    })
  }, [load])

  const submit = async () => {
    if (!orgID) return
    await createProject(Number(orgID), name)
    setName('')
    await load()
  }

  return (
    <Card withBorder>
      <Stack>
        <Title order={4}>Projects (1 project = 1 git repository)</Title>
        <Group align="end">
          <Select
            label="Organization"
            data={orgs.map((org) => ({ value: String(org.id), label: org.name }))}
            value={orgID}
            onChange={setOrgID}
          />
          <TextInput label="Project name" value={name} onChange={(e) => setName(e.currentTarget.value)} />
          <Button onClick={submit}>Create project</Button>
        </Group>
        {projects.map((project) => (
          <Card key={project.id} withBorder radius="md">
            <Stack gap="xs">
              <Group justify="space-between">
                <Text fw={600}>
                  #{project.id} {project.name}
                </Text>
                <Group>
                  {project.archived && <Badge color="gray">archived</Badge>}
                  <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/repo`)}>
                    Repository
                  </Button>
                  <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/issues`)}>
                    Issues
                  </Button>
                  <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/merge-requests`)}>
                    Merge requests
                  </Button>
                </Group>
              </Group>
              <Text size="sm">{project.description || 'No description'}</Text>
              <Text size="sm">Default branch: {project.defaultBranch || 'main'}</Text>
              <Text size="sm">SSH: ssh://git@localhost:2222/{project.repoPath}</Text>
              <Text size="sm">HTTP: http://localhost:8080/git/{project.repoPath}</Text>
            </Stack>
          </Card>
        ))}
      </Stack>
    </Card>
  )
}

function ProjectNav({ projectId, current }: { projectId: number; current: 'repo' | 'issues' | 'merge-requests' }) {
  const items = [
    { key: 'repo', label: 'Repository', to: `/projects/${projectId}/repo` },
    { key: 'issues', label: 'Issues', to: `/projects/${projectId}/issues` },
    { key: 'merge-requests', label: 'Merge requests', to: `/projects/${projectId}/merge-requests` },
  ] as const

  return (
    <Group justify="space-between">
      <Group>
        {items.map((item) => (
          <Button key={item.key} component={Link} to={item.to} variant={current === item.key ? 'filled' : 'light'}>
            {item.label}
          </Button>
        ))}
      </Group>
      <Button component={Link} to="/" variant="subtle">
        Back
      </Button>
    </Group>
  )
}

function RepositoryBrowser() {
  useSignals()
  const session = sessionSignal.value
  const { projectId } = useParams()
  const pid = Number(projectId)
  const [project, setProject] = useState<Project | null>(null)
  const [branches, setBranches] = useState<RepoBranch[]>([])
  const [branch, setBranch] = useState('')
  const [currentPath, setCurrentPath] = useState('')
  const [entries, setEntries] = useState<RepoEntry[]>([])
  const [editorPath, setEditorPath] = useState('')
  const [editorContent, setEditorContent] = useState('')
  const [newBranchName, setNewBranchName] = useState('')
  const [newBranchSource, setNewBranchSource] = useState('')
  const [tags, setTags] = useState<RepoTag[]>([])
  const [tagName, setTagName] = useState('')
  const [tagTarget, setTagTarget] = useState('')
  const [commits, setCommits] = useState<RepoCommit[]>([])
  const [selectedCommit, setSelectedCommit] = useState<RepoCommitDetails | null>(null)
  const [blame, setBlame] = useState<RepoBlameLine[]>([])
  const [settingsDescription, setSettingsDescription] = useState('')
  const [settingsDefaultBranch, setSettingsDefaultBranch] = useState('')

  const branchOptions = useMemo(() => branches.map((item) => item.name), [branches])
  const commitPath = editorPath || currentPath

  const loadProject = useCallback(async () => {
    const data = await getProjectSettings(pid)
    setProject(data)
    setSettingsDescription(data.description || '')
    setSettingsDefaultBranch(data.defaultBranch || 'main')
  }, [pid])

  const loadBranches = useCallback(async () => {
    const data = await listRepoBranches(pid)
    setBranches(data)
    const defaultBranch = data.find((item) => item.isDefault)?.name || data[0]?.name || 'main'
    setBranch((current) => current || defaultBranch)
    setNewBranchSource((current) => current || defaultBranch)
    setTagTarget((current) => current || defaultBranch)
  }, [pid])

  const loadTree = useCallback(async () => {
    if (!Number.isFinite(pid) || !branch) return
    setEntries(await listRepoTree(pid, branch, currentPath))
  }, [branch, currentPath, pid])

  const loadTags = useCallback(async () => {
    if (!Number.isFinite(pid)) return
    setTags(await listRepoTags(pid))
  }, [pid])

  const loadCommits = useCallback(async () => {
    if (!Number.isFinite(pid) || !branch) return
    setCommits(await listRepoCommits(pid, branch, commitPath))
  }, [branch, commitPath, pid])

  useEffect(() => {
    if (!Number.isFinite(pid)) return
    queueMicrotask(() => {
      Promise.all([loadProject(), loadBranches(), loadTags()]).catch(() => undefined)
    })
  }, [loadBranches, loadProject, loadTags, pid])

  useEffect(() => {
    if (!branch) return
    queueMicrotask(() => {
      Promise.all([loadTree(), loadCommits()]).catch(() => undefined)
    })
  }, [branch, loadCommits, loadTree])

  useEffect(() => {
    if (!branch || !editorPath) return
    queueMicrotask(() => {
      getRepoBlame(pid, branch, editorPath)
        .then(setBlame)
        .catch(() => setBlame([]))
    })
  }, [branch, editorPath, pid])

  const openFile = async (path: string) => {
    const file = await getRepoFile(pid, branch, path)
    setEditorPath(file.path)
    setEditorContent(file.content)
  }

  const save = async () => {
    if (!editorPath.trim() || !branch.trim()) return
    await saveRepoFile(pid, { branch, path: editorPath.trim(), content: editorContent })
    notifications.show({ title: 'File saved', message: `${editorPath.trim()} updated on ${branch}.` })
    await Promise.all([loadProject(), loadBranches(), loadTree(), loadCommits()])
    await openFile(editorPath.trim())
  }

  const remove = async () => {
    if (!editorPath.trim()) return
    if (!window.confirm(`Delete ${editorPath.trim()}?`)) return
    await deleteRepoFile(pid, { branch, path: editorPath.trim() })
    notifications.show({ title: 'File deleted', message: `${editorPath.trim()} removed from ${branch}.` })
    setEditorPath('')
    setEditorContent('')
    setBlame([])
    await Promise.all([loadTree(), loadCommits()])
  }

  const createBranch = async () => {
    if (!newBranchName.trim() || !newBranchSource.trim()) return
    await createRepoBranch(pid, newBranchName.trim(), newBranchSource.trim())
    setNewBranchName('')
    await loadBranches()
  }

  const removeBranch = async () => {
    if (!branch.trim()) return
    await deleteRepoBranch(pid, branch.trim())
    setEditorPath('')
    setEditorContent('')
    await loadBranches()
    await loadTree()
  }

  const createTag = async () => {
    if (!tagName.trim()) return
    await createRepoTag(pid, tagName.trim(), tagTarget.trim())
    setTagName('')
    await loadTags()
  }

  const loadCommit = async (hash: string) => {
    setSelectedCommit(await getRepoCommit(pid, hash))
  }

  const saveSettings = async () => {
    const updated = await updateProjectSettings(pid, {
      description: settingsDescription,
      defaultBranch: settingsDefaultBranch,
    })
    setProject(updated)
    notifications.show({ title: 'Repository settings updated', message: `${updated.name} settings saved.` })
    await loadBranches()
  }

  const toggleArchive = async () => {
    if (!project) return
    const updated = await updateProjectSettings(pid, { archived: !project.archived })
    setProject(updated)
    notifications.show({
      title: updated.archived ? 'Repository archived' : 'Repository unarchived',
      message: `${updated.name} is now ${updated.archived ? 'read only' : 'writable'}.`,
    })
  }

  const segments = currentPath ? currentPath.split('/') : []

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Project #{pid} repository</Title>
        <ProjectNav projectId={pid} current="repo" />
      </Group>

      {project && (
        <Card withBorder>
          <Stack>
            <Group justify="space-between">
              <Title order={4}>Repository settings</Title>
              {project.archived && <Badge color="gray">archived</Badge>}
            </Group>
            <TextInput
              label="Description"
              value={settingsDescription}
              onChange={(e) => setSettingsDescription(e.currentTarget.value)}
            />
            <Autocomplete
              label="Default branch"
              data={branchOptions}
              value={settingsDefaultBranch}
              onChange={setSettingsDefaultBranch}
            />
            <Group>
              <Button onClick={saveSettings} disabled={!session}>
                Save settings
              </Button>
              <Button variant="light" color={project.archived ? 'green' : 'gray'} onClick={toggleArchive} disabled={!session}>
                {project.archived ? 'Unarchive' : 'Archive'}
              </Button>
            </Group>
          </Stack>
        </Card>
      )}

      <Card withBorder>
        <Stack>
          <Title order={4}>Branches and tags</Title>
          <Group align="end">
            <Autocomplete label="Current branch" data={branchOptions} value={branch} onChange={setBranch} />
            <Button variant="light" color="red" onClick={removeBranch} disabled={!session || branches.find((item) => item.name === branch)?.isDefault}>
              Delete branch
            </Button>
          </Group>
          <Group align="end">
            <TextInput label="New branch" value={newBranchName} onChange={(e) => setNewBranchName(e.currentTarget.value)} />
            <Autocomplete label="From branch" data={branchOptions} value={newBranchSource} onChange={setNewBranchSource} />
            <Button onClick={createBranch} disabled={!session || !newBranchName.trim()}>
              Create branch
            </Button>
          </Group>
          <Group align="end">
            <TextInput label="New tag" value={tagName} onChange={(e) => setTagName(e.currentTarget.value)} />
            <Autocomplete label="Target" data={branchOptions} value={tagTarget} onChange={setTagTarget} />
            <Button onClick={createTag} disabled={!session || !tagName.trim()}>
              Create tag
            </Button>
          </Group>
          {tags.length === 0 && <Text size="sm">No tags yet.</Text>}
          {tags.map((tag) => (
            <Text key={tag.name} size="sm">
              {tag.name} → {tag.target.slice(0, 12)}
            </Text>
          ))}
        </Stack>
      </Card>

      <Card withBorder>
        <Stack>
          <Group align="end">
            <Autocomplete label="Branch" data={branchOptions} value={branch} onChange={setBranch} />
            <Button variant="light" onClick={() => setCurrentPath('')}>
              Root
            </Button>
            <Button
              variant="light"
              onClick={() => {
                setEditorPath(currentPath ? `${currentPath}/new-file.txt` : 'new-file.txt')
                setEditorContent('')
              }}
            >
              New file
            </Button>
          </Group>
          <Group gap="xs">
            <Button variant={currentPath === '' ? 'filled' : 'light'} size="xs" onClick={() => setCurrentPath('')}>
              /
            </Button>
            {segments.map((segment, index) => {
              const nextPath = segments.slice(0, index + 1).join('/')
              return (
                <Button
                  key={nextPath}
                  variant={currentPath === nextPath ? 'filled' : 'light'}
                  size="xs"
                  onClick={() => setCurrentPath(nextPath)}
                >
                  {segment}
                </Button>
              )
            })}
          </Group>
          {entries.length === 0 && <Text size="sm">No files at this location yet.</Text>}
          {entries.map((entry) => (
            <Card key={entry.path} withBorder>
              <Group justify="space-between">
                <Text>
                  {entry.type === 'dir' ? '📁' : '📄'} {entry.name}
                </Text>
                <Button size="xs" variant="subtle" onClick={() => (entry.type === 'dir' ? setCurrentPath(entry.path) : openFile(entry.path))}>
                  {entry.type === 'dir' ? 'Open' : 'Edit'}
                </Button>
              </Group>
            </Card>
          ))}
        </Stack>
      </Card>

      <Card withBorder>
        <Stack>
          <Title order={4}>Editor</Title>
          <TextInput
            label="File path"
            value={editorPath}
            onChange={(e) => setEditorPath(e.currentTarget.value)}
            placeholder="src/main.txt"
          />
          <Textarea
            label="Content"
            minRows={18}
            value={editorContent}
            onChange={(e) => setEditorContent(e.currentTarget.value)}
            placeholder="File content"
          />
          <Group>
            <Button onClick={save} disabled={!session || !branch.trim() || !editorPath.trim() || project?.archived}>
              Save file
            </Button>
            <Button variant="light" color="red" onClick={remove} disabled={!session || !editorPath.trim() || project?.archived}>
              Delete file
            </Button>
          </Group>
          {!session && <Text size="sm">Sign in first to edit or delete files.</Text>}
        </Stack>
      </Card>

      <Card withBorder>
        <Stack>
          <Title order={4}>Commit history</Title>
          {commits.length === 0 && <Text size="sm">No commits yet.</Text>}
          {commits.map((commit) => (
            <Card key={commit.hash} withBorder>
              <Group justify="space-between">
                <Stack gap={2}>
                  <Text fw={700}>
                    {commit.shortHash} {commit.subject}
                  </Text>
                  <Text size="sm">
                    {commit.authorName} · {new Date(commit.authoredAt).toLocaleString()}
                  </Text>
                </Stack>
                <Button size="xs" variant="light" onClick={() => loadCommit(commit.hash)}>
                  Show
                </Button>
              </Group>
            </Card>
          ))}
          {selectedCommit && (
            <Card withBorder>
              <Stack>
                <Title order={5}>
                  {selectedCommit.shortHash} {selectedCommit.subject}
                </Title>
                <Text size="sm">{selectedCommit.authorName}</Text>
                <Textarea readOnly autosize minRows={10} value={selectedCommit.diff || 'No diff available.'} />
              </Stack>
            </Card>
          )}
        </Stack>
      </Card>

      {editorPath && (
        <Card withBorder>
          <Stack>
            <Title order={4}>Blame for {editorPath}</Title>
            {blame.length === 0 && <Text size="sm">No blame data available.</Text>}
            {blame.slice(0, 20).map((line) => (
              <Text key={`${line.lineNumber}-${line.commitHash}`} size="sm">
                {line.lineNumber}. {line.commitHash.slice(0, 8)} · {line.authorName} · {line.content}
              </Text>
            ))}
          </Stack>
        </Card>
      )}
    </Stack>
  )
}

type IssueStatus = Issue['status']

const ISSUE_COLUMNS: { status: IssueStatus; title: string; color: string }[] = [
  { status: 'open', title: 'Open', color: 'green' },
  { status: 'closed', title: 'Closed', color: 'gray' },
]

function parseIssueTags(input: string): string[] {
  const seen = new Set<string>()
  const tags: string[] = []
  for (const rawTag of input.split(/[,\n;]+/)) {
    const tag = rawTag.trim()
    if (!tag || seen.has(tag)) continue
    seen.add(tag)
    tags.push(tag)
  }
  return tags
}

function IssueColumn({
  status,
  title,
  color,
  children,
}: {
  status: IssueStatus
  title: string
  color: string
  children: ReactNode
}) {
  const { isOver, setNodeRef } = useDroppable({ id: status })
  return (
    <Card ref={setNodeRef} withBorder style={{ width: '100%', backgroundColor: isOver ? '#eef7ff' : undefined, minHeight: 220 }}>
      <Stack>
        <Group justify="space-between">
          <Title order={5}>{title}</Title>
          <Badge color={color}>{status}</Badge>
        </Group>
        {children}
      </Stack>
    </Card>
  )
}

function DraggableIssueCard({
  issue,
  onToggleStatus,
  tagValue,
  onTagChange,
  onSaveTags,
  commentValue,
  onCommentChange,
  onAddComment,
}: {
  issue: Issue
  onToggleStatus: (issue: Issue) => void
  tagValue: string
  onTagChange: (value: string) => void
  onSaveTags: () => void
  commentValue: string
  onCommentChange: (value: string) => void
  onAddComment: () => void
}) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
    id: `issue-${issue.id}`,
    data: { issueId: issue.id, status: issue.status },
  })
  const style = {
    transform: CSS.Translate.toString(transform),
    opacity: isDragging ? 0.6 : 1,
  }

  return (
    <Card ref={setNodeRef} withBorder style={style}>
      <Stack>
        <Group justify="space-between">
          <Text fw={700}>
            #{issue.id} {issue.title}
          </Text>
          <Group>
            <Badge color={issue.status === 'open' ? 'green' : 'gray'}>{issue.status}</Badge>
            <ActionIcon variant="light" aria-label={`Toggle status for issue ${issue.id}`} onClick={() => onToggleStatus(issue)}>
              ↻
            </ActionIcon>
            <ActionIcon variant="subtle" aria-label={`Drag issue ${issue.id}`} {...attributes} {...listeners}>
              ⋮⋮
            </ActionIcon>
          </Group>
        </Group>
        <Text>{issue.description}</Text>
        <Group gap="xs">
          {issue.tags.length === 0 && (
            <Badge variant="light" color="gray">
              no tags
            </Badge>
          )}
          {issue.tags.map((tag) => (
            <Badge key={tag} variant="light">
              {tag}
            </Badge>
          ))}
        </Group>
        <Group align="end">
          <TextInput
            label="Tags"
            aria-label={`Tags for issue ${issue.id}`}
            placeholder="bug, docs"
            value={tagValue}
            onChange={(e) => onTagChange(e.currentTarget.value)}
          />
          <Button size="xs" variant="light" onClick={onSaveTags}>
            Save tags
          </Button>
        </Group>
        {issue.comments.map((comment) => (
          <Text size="sm" key={comment.id}>
            • {comment.body}
          </Text>
        ))}
        <Group align="end">
          <TextInput
            aria-label={`Comment for issue ${issue.id}`}
            placeholder="Add comment"
            value={commentValue}
            onChange={(e) => onCommentChange(e.currentTarget.value)}
          />
          <Button size="xs" onClick={onAddComment}>
            Comment
          </Button>
        </Group>
      </Stack>
    </Card>
  )
}

function IssueBoard() {
  const { projectId } = useParams()
  const pid = Number(projectId)
  const [issues, setIssues] = useState<Issue[]>([])
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [newIssueTags, setNewIssueTags] = useState('')
  const [commentBody, setCommentBody] = useState<Record<number, string>>({})
  const [tagDrafts, setTagDrafts] = useState<Record<number, string>>({})
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 8 } }), useSensor(KeyboardSensor))

  const load = useCallback(async () => {
    const data = await listIssues(pid)
    setIssues(data)
    setTagDrafts(Object.fromEntries(data.map((issue) => [issue.id, issue.tags.join(', ')])))
  }, [pid])

  useEffect(() => {
    if (Number.isFinite(pid)) {
      queueMicrotask(() => {
        load().catch(() => undefined)
      })
    }
  }, [load, pid])

  const create = async () => {
    await createIssue(pid, title, description, parseIssueTags(newIssueTags))
    setTitle('')
    setDescription('')
    setNewIssueTags('')
    await load()
  }

  const toggleStatus = async (issue: Issue) => {
    await updateIssue(pid, issue.id, { status: issue.status === 'open' ? 'closed' : 'open' })
    await load()
  }

  const saveTags = async (issueId: number) => {
    await updateIssue(pid, issueId, { tags: parseIssueTags(tagDrafts[issueId] ?? '') })
    await load()
  }

  const addComment = async (issueId: number) => {
    const body = commentBody[issueId]?.trim()
    if (!body) return
    await addIssueComment(pid, issueId, body)
    setCommentBody((curr) => ({ ...curr, [issueId]: '' }))
    await load()
  }

  const handleDragEnd = async (event: DragEndEvent) => {
    const over = event.over?.id
    if (over !== 'open' && over !== 'closed') return
    const issueId = Number(event.active.data.current?.issueId)
    if (!Number.isFinite(issueId)) return
    const issue = issues.find((item) => item.id === issueId)
    if (!issue || issue.status === over) return
    await updateIssue(pid, issue.id, { status: over })
    await load()
  }

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Project #{pid} issues</Title>
        <ProjectNav projectId={pid} current="issues" />
      </Group>

      <Card withBorder>
        <Stack>
          <TextInput label="Title" value={title} onChange={(e) => setTitle(e.currentTarget.value)} />
          <Textarea label="Description" value={description} onChange={(e) => setDescription(e.currentTarget.value)} />
          <TextInput label="Tags" placeholder="bug, docs" value={newIssueTags} onChange={(e) => setNewIssueTags(e.currentTarget.value)} />
          <Button onClick={create}>Open issue</Button>
        </Stack>
      </Card>

      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <Group align="start" grow>
          {ISSUE_COLUMNS.map((column) => {
            const columnIssues = issues.filter((issue) => issue.status === column.status)
            return (
              <IssueColumn key={column.status} status={column.status} title={column.title} color={column.color}>
                <Stack>
                  {columnIssues.map((issue) => (
                    <DraggableIssueCard
                      key={issue.id}
                      issue={issue}
                      onToggleStatus={toggleStatus}
                      tagValue={tagDrafts[issue.id] ?? ''}
                      onTagChange={(value) => setTagDrafts((curr) => ({ ...curr, [issue.id]: value }))}
                      onSaveTags={() => saveTags(issue.id)}
                      commentValue={commentBody[issue.id] ?? ''}
                      onCommentChange={(value) => setCommentBody((curr) => ({ ...curr, [issue.id]: value }))}
                      onAddComment={() => addComment(issue.id)}
                    />
                  ))}
                  {columnIssues.length === 0 && <Text size="sm">No issues.</Text>}
                </Stack>
              </IssueColumn>
            )
          })}
        </Group>
      </DndContext>
    </Stack>
  )
}

function MergeRequestBoard() {
  const { projectId } = useParams()
  const pid = Number(projectId)
  const navigate = useNavigate()
  const [mergeRequests, setMergeRequests] = useState<MergeRequest[]>([])
  const [branches, setBranches] = useState<RepoBranch[]>([])
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [sourceBranch, setSourceBranch] = useState('')
  const [targetBranch, setTargetBranch] = useState('')

  const branchOptions = useMemo(() => branches.map((item) => item.name), [branches])

  const load = useCallback(async () => {
    const [mrs, repoBranches] = await Promise.all([listMergeRequests(pid), listRepoBranches(pid)])
    setMergeRequests(mrs)
    setBranches(repoBranches)
    setTargetBranch((current) => current || repoBranches.find((item) => item.isDefault)?.name || repoBranches[0]?.name || 'main')
    setSourceBranch((current) => current || repoBranches.find((item) => !item.isDefault)?.name || repoBranches[0]?.name || 'main')
  }, [pid])

  useEffect(() => {
    if (!Number.isFinite(pid)) return
    queueMicrotask(() => {
      load().catch(() => undefined)
    })
  }, [load, pid])

  const create = async () => {
    const mr = await createMergeRequest(pid, title, description, sourceBranch, targetBranch)
    setTitle('')
    setDescription('')
    notifications.show({ title: 'Merge request created', message: `MR #${mr.id} is ready for review.` })
    await load()
    navigate(`/projects/${pid}/merge-requests/${mr.id}`)
  }

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Project #{pid} merge requests</Title>
        <ProjectNav projectId={pid} current="merge-requests" />
      </Group>

      <Card withBorder>
        <Stack>
          <Title order={4}>Open merge request</Title>
          <TextInput label="Title" value={title} onChange={(e) => setTitle(e.currentTarget.value)} />
          <Textarea label="Description" value={description} onChange={(e) => setDescription(e.currentTarget.value)} />
          <Group align="end">
            <Autocomplete label="Source branch" data={branchOptions} value={sourceBranch} onChange={setSourceBranch} />
            <Autocomplete label="Target branch" data={branchOptions} value={targetBranch} onChange={setTargetBranch} />
            <Button onClick={create}>Create</Button>
          </Group>
        </Stack>
      </Card>

      {mergeRequests.length === 0 && <Text size="sm">No merge requests yet.</Text>}
      {mergeRequests.map((mr) => (
        <Card key={mr.id} withBorder>
          <Stack gap="xs">
            <Group justify="space-between">
              <Text fw={700}>
                #{mr.id} {mr.title}
              </Text>
              <Group>
                <Badge color={mr.status === 'merged' ? 'green' : mr.hasConflicts ? 'red' : 'blue'}>{mr.status}</Badge>
                {mr.mergeable && <Badge color="teal">mergeable</Badge>}
                {mr.hasConflicts && <Badge color="red">conflicts</Badge>}
              </Group>
            </Group>
            <Text size="sm">
              {mr.sourceBranch} → {mr.targetBranch}
            </Text>
            <Text size="sm">{mr.description || 'No description'}</Text>
            <Text size="sm">Comments: {mr.comments.length}</Text>
            <Button component={Link} to={`/projects/${pid}/merge-requests/${mr.id}`} size="xs" variant="light">
              View
            </Button>
          </Stack>
        </Card>
      ))}
    </Stack>
  )
}

function MergeRequestDetails() {
  useSignals()
  const session = sessionSignal.value
  const { projectId, mergeRequestId } = useParams()
  const pid = Number(projectId)
  const mrid = Number(mergeRequestId)
  const [mergeRequest, setMergeRequest] = useState<MergeRequest | null>(null)
  const [diff, setDiff] = useState('')
  const [comment, setComment] = useState('')

  const load = useCallback(async () => {
    const [mr, diffResponse, mergeStatus] = await Promise.all([
      getMergeRequest(pid, mrid),
      getMergeRequestDiff(pid, mrid),
      getMergeRequestMergeStatus(pid, mrid),
    ])
    setMergeRequest({ ...mr, ...mergeStatus })
    setDiff(diffResponse.diff)
  }, [mrid, pid])

  useEffect(() => {
    if (!Number.isFinite(pid) || !Number.isFinite(mrid)) return
    queueMicrotask(() => {
      load().catch(() => undefined)
    })
  }, [load, mrid, pid])

  const addComment = async () => {
    if (!comment.trim()) return
    await addMergeRequestComment(pid, mrid, comment.trim())
    setComment('')
    await load()
  }

  const merge = async () => {
    const mr = await mergeMergeRequest(pid, mrid)
    setMergeRequest(mr)
    notifications.show({ title: 'Merge request merged', message: `MR #${mr.id} merged into ${mr.targetBranch}.` })
    await load()
  }

  if (!mergeRequest) return <Text>Loading merge request…</Text>

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>
          MR #{mergeRequest.id} {mergeRequest.title}
        </Title>
        <ProjectNav projectId={pid} current="merge-requests" />
      </Group>

      <Card withBorder>
        <Stack gap="xs">
          <Group>
            <Badge color={mergeRequest.status === 'merged' ? 'green' : mergeRequest.hasConflicts ? 'red' : 'blue'}>
              {mergeRequest.status}
            </Badge>
            {mergeRequest.mergeable && <Badge color="teal">mergeable</Badge>}
            {mergeRequest.hasConflicts && <Badge color="red">conflicts</Badge>}
            {mergeRequest.alreadyMerged && <Badge color="green">already merged</Badge>}
            <Text size="sm">
              {mergeRequest.sourceBranch} → {mergeRequest.targetBranch}
            </Text>
          </Group>
          <Text>{mergeRequest.description || 'No description'}</Text>
          {mergeRequest.mergedCommitId && <Text size="sm">Merge commit: {mergeRequest.mergedCommitId}</Text>}
          <Button onClick={merge} disabled={!session || !mergeRequest.mergeable || mergeRequest.status !== 'open'}>
            Merge request
          </Button>
          {!session && <Text size="sm">Sign in first to merge.</Text>}
        </Stack>
      </Card>

      <Card withBorder>
        <Stack>
          <Title order={4}>Diff</Title>
          <Textarea readOnly autosize minRows={12} value={diff || 'No diff available.'} />
        </Stack>
      </Card>

      <Card withBorder>
        <Stack>
          <Title order={4}>Comments</Title>
          {mergeRequest.comments.length === 0 && <Text size="sm">No comments yet.</Text>}
          {mergeRequest.comments.map((item) => (
            <Card key={item.id} withBorder>
              <Stack gap="xs">
                <Text size="sm">User #{item.authorId}</Text>
                <Text>{item.body}</Text>
              </Stack>
            </Card>
          ))}
          <Textarea label="New comment" minRows={3} value={comment} onChange={(e) => setComment(e.currentTarget.value)} />
          <Button onClick={addComment} disabled={!session}>
            Add comment
          </Button>
        </Stack>
      </Card>
    </Stack>
  )
}

export default App
