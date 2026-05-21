import {
  ActionIcon,
  AppShell,
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
import { useSignal } from '@preact/signals-react/runtime'
import { useEffect, useState } from 'react'
import { Link, Navigate, Route, Routes, useNavigate, useParams } from 'react-router-dom'
import {
  addIssueComment,
  addSSHKey,
  createIssue,
  createOrganization,
  createProject,
  createUser,
  listIssues,
  listOrganizations,
  listProjects,
  updateIssue,
  type Issue,
  type Organization,
  type Project,
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
              <Route path="/projects/:projectId/issues" element={<IssueBoard />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  )
}

function AuthPill() {
  const session = useSignal(sessionSignal)
  return session.value ? (
    <Badge variant="light">{session.value.username}</Badge>
  ) : (
    <Badge color="gray">Guest</Badge>
  )
}

function Home() {
  const session = useSignal(sessionSignal)
  return (
    <Stack>
      {!session.value && <Onboard />}
      {session.value && (
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
  const session = useSignal(sessionSignal)
  const [key, setKey] = useState('')
  const submit = async () => {
    if (!session.value) return
    await addSSHKey(session.value.userId, key)
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
  const load = () =>
    listOrganizations()
      .then(setOrgs)
      .catch(() => undefined)
  useEffect(() => {
    load()
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

  const load = async () => {
    const [projectData, orgData] = await Promise.all([listProjects(), listOrganizations()])
    setProjects(projectData)
    setOrgs(orgData)
    if (!orgID && orgData.length > 0) setOrgID(String(orgData[0].id))
  }
  useEffect(() => {
    load().catch(() => undefined)
  }, [])

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
                <Button size="xs" variant="light" onClick={() => navigate(`/projects/${project.id}/issues`)}>
                  Issues
                </Button>
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

function IssueBoard() {
  const { projectId } = useParams()
  const pid = Number(projectId)
  const [issues, setIssues] = useState<Issue[]>([])
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [commentBody, setCommentBody] = useState<Record<number, string>>({})

  const load = async () => {
    setIssues(await listIssues(pid))
  }
  useEffect(() => {
    if (Number.isFinite(pid)) load().catch(() => undefined)
  }, [pid])

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
        <Button component={Link} to="/" variant="subtle">
          Back
        </Button>
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

export default App
