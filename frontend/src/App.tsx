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
import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link, Navigate, Route, Routes, useNavigate, useParams } from 'react-router-dom'
import {
  addIssueComment,
  addMergeRequestComment,
  addSSHKey,
  createIssue,
  createMergeRequest,
  createOrganization,
  createProject,
  createUser,
  deleteRepoFile,
  getMergeRequest,
  getMergeRequestDiff,
  getRepoFile,
  listIssues,
  listMergeRequests,
  listOrganizations,
  listProjects,
  listRepoBranches,
  listRepoTree,
  saveRepoFile,
  updateIssue,
  type Issue,
  type MergeRequest,
  type Organization,
  type Project,
  type RepoBranch,
  type RepoEntry,
} from './api'
import { sessionSignal } from './state'

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
        <Textarea
          value={key}
          minRows={3}
          onChange={(e) => setKey(e.currentTarget.value)}
          placeholder="ssh-ed25519 AAAA..."
        />
        <Button onClick={submit}>Add SSH key</Button>
      </Stack>
    </Card>
  )
}

function OrganizationsPanel() {
  const [name, setName] = useState('')
  const [orgs, setOrgs] = useState<Organization[]>([])
  const load = () => listOrganizations().then(setOrgs).catch(() => undefined)
  useEffect(() => {
    queueMicrotask(() => {
      load()
    })
  }, [])
  const create = async () => {
    await createOrganization(name)
    setName('')
    await load()
  }
  return (
    <Card withBorder>
      <Stack>
        <Title order={4}>Organizations</Title>
        <Group>
          <TextInput value={name} onChange={(e) => setName(e.currentTarget.value)} placeholder="acme" />
          <Button onClick={create}>Create</Button>
        </Group>
        {orgs.map((org) => (
          <Text key={org.id}>
            #{org.id} {org.name}
          </Text>
        ))}
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
                  <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/repo`)}>
                    Repository
                  </Button>
                  <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/issues`)}>
                    Issues
                  </Button>
                  <Button
                    size="xs"
                    variant="light"
                    onClick={() => navigate(`/projects/${project.id}/merge-requests`)}
                  >
                    Merge requests
                  </Button>
                </Group>
              </Group>
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
  const [branches, setBranches] = useState<RepoBranch[]>([])
  const [branch, setBranch] = useState('')
  const [currentPath, setCurrentPath] = useState('')
  const [entries, setEntries] = useState<RepoEntry[]>([])
  const [editorPath, setEditorPath] = useState('')
  const [editorContent, setEditorContent] = useState('')

  const branchOptions = useMemo(() => branches.map((item) => item.name), [branches])

  const loadBranches = useCallback(async () => {
    const data = await listRepoBranches(pid)
    setBranches(data)
    setBranch((current) => current || data.find((item) => item.isDefault)?.name || data[0]?.name || 'main')
  }, [pid])

  const loadTree = useCallback(async () => {
    if (!Number.isFinite(pid) || !branch) return
    const data = await listRepoTree(pid, branch, currentPath)
    setEntries(data)
  }, [branch, currentPath, pid])

  useEffect(() => {
    if (!Number.isFinite(pid)) return
    queueMicrotask(() => {
      loadBranches().catch(() => undefined)
    })
  }, [loadBranches, pid])

  useEffect(() => {
    if (!branch) return
    queueMicrotask(() => {
      loadTree().catch(() => undefined)
    })
  }, [branch, loadTree])

  const openFile = async (path: string) => {
    const file = await getRepoFile(pid, branch, path)
    setEditorPath(file.path)
    setEditorContent(file.content)
  }

  const save = async () => {
    if (!editorPath.trim() || !branch.trim()) return
    await saveRepoFile(pid, { branch, path: editorPath.trim(), content: editorContent })
    notifications.show({ title: 'File saved', message: `${editorPath.trim()} updated on ${branch}.` })
    await loadBranches()
    await loadTree()
    await openFile(editorPath.trim())
  }

  const remove = async () => {
    if (!editorPath.trim()) return
    if (!window.confirm(`Delete ${editorPath.trim()}?`)) return
    await deleteRepoFile(pid, { branch, path: editorPath.trim() })
    notifications.show({ title: 'File deleted', message: `${editorPath.trim()} removed from ${branch}.` })
    setEditorPath('')
    setEditorContent('')
    await loadTree()
  }

  const segments = currentPath ? currentPath.split('/') : []

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Project #{pid} repository</Title>
        <ProjectNav projectId={pid} current="repo" />
      </Group>

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
                <Button
                  size="xs"
                  variant="subtle"
                  onClick={() => (entry.type === 'dir' ? setCurrentPath(entry.path) : openFile(entry.path))}
                >
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
            <Button onClick={save} disabled={!session || !branch.trim() || !editorPath.trim()}>
              Save file
            </Button>
            <Button variant="light" color="red" onClick={remove} disabled={!session || !editorPath.trim()}>
              Delete file
            </Button>
          </Group>
          {!session && <Text size="sm">Sign in first to edit or delete files.</Text>}
        </Stack>
      </Card>
    </Stack>
  )
}

function IssueBoard() {
  const { projectId } = useParams()
  const pid = Number(projectId)
  const [issues, setIssues] = useState<Issue[]>([])
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [commentBody, setCommentBody] = useState<Record<number, string>>({})

  const load = useCallback(async () => {
    setIssues(await listIssues(pid))
  }, [pid])
  useEffect(() => {
    if (Number.isFinite(pid)) {
      queueMicrotask(() => {
        load().catch(() => undefined)
      })
    }
  }, [pid, load])

  const create = async () => {
    await createIssue(pid, title, description)
    setTitle('')
    setDescription('')
    await load()
  }

  const toggleStatus = async (issue: Issue) => {
    await updateIssue(pid, issue.id, { status: issue.status === 'open' ? 'closed' : 'open' })
    await load()
  }

  const addComment = async (issueId: number) => {
    const body = commentBody[issueId]?.trim()
    if (!body) return
    await addIssueComment(pid, issueId, body)
    setCommentBody((curr) => ({ ...curr, [issueId]: '' }))
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
          <Textarea
            label="Description"
            value={description}
            onChange={(e) => setDescription(e.currentTarget.value)}
          />
          <Button onClick={create}>Open issue</Button>
        </Stack>
      </Card>

      {issues.map((issue) => (
        <Card key={issue.id} withBorder>
          <Stack>
            <Group justify="space-between">
              <Text fw={700}>
                #{issue.id} {issue.title}
              </Text>
              <Group>
                <Badge color={issue.status === 'open' ? 'green' : 'gray'}>{issue.status}</Badge>
                <ActionIcon variant="light" onClick={() => toggleStatus(issue)}>
                  ↻
                </ActionIcon>
              </Group>
            </Group>
            <Text>{issue.description}</Text>
            {issue.comments.map((comment) => (
              <Text size="sm" key={comment.id}>
                • {comment.body}
              </Text>
            ))}
            <Group align="end">
              <TextInput
                placeholder="Add comment"
                value={commentBody[issue.id] ?? ''}
                onChange={(e) =>
                  setCommentBody((curr) => ({ ...curr, [issue.id]: e.currentTarget.value }))
                }
              />
              <Button size="xs" onClick={() => addComment(issue.id)}>
                Comment
              </Button>
            </Group>
          </Stack>
        </Card>
      ))}
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
          <Textarea
            label="Description"
            value={description}
            onChange={(e) => setDescription(e.currentTarget.value)}
          />
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
              <Badge color={mr.status === 'open' ? 'blue' : 'gray'}>{mr.status}</Badge>
            </Group>
            <Text size="sm">
              {mr.sourceBranch} → {mr.targetBranch}
            </Text>
            <Text size="sm">{mr.description || 'No description'}</Text>
            <Text size="sm">Comments: {mr.comments.length}</Text>
            <Group>
              <Button component={Link} to={`/projects/${pid}/merge-requests/${mr.id}`} size="xs" variant="light">
                View
              </Button>
            </Group>
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
    const [mr, diffResponse] = await Promise.all([getMergeRequest(pid, mrid), getMergeRequestDiff(pid, mrid)])
    setMergeRequest(mr)
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
            <Badge color={mergeRequest.status === 'open' ? 'blue' : 'gray'}>{mergeRequest.status}</Badge>
            <Text size="sm">
              {mergeRequest.sourceBranch} → {mergeRequest.targetBranch}
            </Text>
          </Group>
          <Text>{mergeRequest.description || 'No description'}</Text>
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
          <Textarea
            label="New comment"
            minRows={3}
            value={comment}
            onChange={(e) => setComment(e.currentTarget.value)}
          />
          <Button onClick={addComment} disabled={!session}>
            Add comment
          </Button>
          {!session && <Text size="sm">Sign in first to leave comments.</Text>}
        </Stack>
      </Card>
    </Stack>
  )
}

export default App
