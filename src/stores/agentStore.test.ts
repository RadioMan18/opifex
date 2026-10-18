import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useAgentStore } from '@/stores/agentStore'

vi.mock('@/services/ipc')

import {
  createAgent,
  getAgent,
  listAgents,
  updateAgent,
  renameAgent,
  duplicateAgent,
  seedStarterAgents,
  ollamaListModels,
  ollamaChatStream,
  ollamaGetDefaultModel,
  ollamaGetProvider,
  ollamaSetProvider,
  ollamaSetApiKey,
  createConversation,
  listConversations,
  savePlanVersion,
  listPlanVersions,
  restorePlanVersion,
} from '@/services/ipc'

const mockCreateAgent = vi.mocked(createAgent)
const mockGetAgent = vi.mocked(getAgent)
const mockListAgents = vi.mocked(listAgents)
const mockUpdateAgent = vi.mocked(updateAgent)
const mockRenameAgent = vi.mocked(renameAgent)
const mockDuplicateAgent = vi.mocked(duplicateAgent)
const mockSeedStarterAgents = vi.mocked(seedStarterAgents)
const mockOllamaListModels = vi.mocked(ollamaListModels)
const mockOllamaChatStream = vi.mocked(ollamaChatStream)
const mockOllamaGetDefaultModel = vi.mocked(ollamaGetDefaultModel)
const mockOllamaGetProvider = vi.mocked(ollamaGetProvider)
const mockOllamaSetProvider = vi.mocked(ollamaSetProvider)
const mockOllamaSetApiKey = vi.mocked(ollamaSetApiKey)
const mockCreateConversation = vi.mocked(createConversation)
const mockListConversations = vi.mocked(listConversations)
const mockSavePlanVersion = vi.mocked(savePlanVersion)
const mockListPlanVersions = vi.mocked(listPlanVersions)
const mockRestorePlanVersion = vi.mocked(restorePlanVersion)

const mockAgent = {
  id: 'test-agent-1',
  name: 'Test Agent',
  description: 'A test agent',
  specJson: { goal: 'Test goal', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] },
  model: 'ollama',
  isStarter: false,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
}

const mockAgentListItem = {
  id: 'test-agent-1',
  name: 'Test Agent',
  description: 'A test agent',
  model: 'ollama',
  isStarter: false,
  updatedAt: '2024-01-01T00:00:00Z',
}

const mockConversation = {
  id: 'conv-1',
  agentId: 'test-agent-1',
  title: 'Test Conversation',
  createdAt: '2024-01-01T00:00:00Z',
}

describe('useAgentStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    const store = useAgentStore()
    store.state.agents = []
    store.state.currentAgent = null
    store.state.currentPlan = null
    store.state.messages = []
    store.state.conversations = []
    store.state.currentConversationId = null
    store.state.planVersions = []
    store.state.models = []
    store.state.defaultModel = null
    store.state.provider = null
    store.state.apiKeys = {}
    store.state.loading = false
    store.state.streaming = false
    store.state.streamingMsgId = null
    store.state.error = null
  })

  describe('init', () => {
    it('should load agents, models, default model, and provider', async () => {
      mockSeedStarterAgents.mockResolvedValue([])
      mockListAgents.mockResolvedValue([mockAgentListItem])
      mockOllamaListModels.mockResolvedValue([{ name: 'llama3.1', size: 1000, modifiedAt: '2024-01-01', digest: 'abc' }])
      mockOllamaGetDefaultModel.mockResolvedValue('llama3.1')
      mockOllamaGetProvider.mockResolvedValue('ollama')

      const store = useAgentStore()
      await store.init()

      expect(mockSeedStarterAgents).toHaveBeenCalled()
      expect(mockListAgents).toHaveBeenCalled()
      expect(mockOllamaListModels).toHaveBeenCalled()
      expect(mockOllamaGetDefaultModel).toHaveBeenCalled()
      expect(mockOllamaGetProvider).toHaveBeenCalled()
      expect(store.state.loading).toBe(false)
    })
  })

  describe('selectAgent', () => {
    it('should set current agent and plan, clear messages', async () => {
      mockGetAgent.mockResolvedValue(mockAgent)
      mockListConversations.mockResolvedValue([])
      mockListPlanVersions.mockResolvedValue([])

      const store = useAgentStore()
      await store.selectAgent('test-agent-1')

      expect(mockGetAgent).toHaveBeenCalledWith('test-agent-1')
      expect(store.state.currentAgent).toEqual(mockAgent)
      expect(store.state.currentPlan).toEqual(mockAgent.specJson)
      expect(store.state.messages).toEqual([])
    })
  })

  describe('createNewAgent', () => {
    it('should create a new agent and set it as current', async () => {
      const newAgent = { ...mockAgent, id: 'new-agent-1', name: 'New Agent' }
      mockCreateAgent.mockResolvedValue(newAgent)
      mockListAgents.mockResolvedValue([mockAgentListItem])

      const store = useAgentStore()
      const result = await store.createNewAgent('New Agent', 'A new agent', 'ollama')

      expect(mockCreateAgent).toHaveBeenCalledWith({
        name: 'New Agent',
        description: 'A new agent',
        specJson: { goal: '', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] },
        model: 'ollama',
        isStarter: false,
      })
      expect(store.state.currentAgent).toEqual(newAgent)
      expect(result).toEqual(newAgent)
    })
  })

  describe('updatePlanField', () => {
    it('should update a plan field and persist it', async () => {
      mockUpdateAgent.mockResolvedValue(mockAgent)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = { goal: 'original', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] }

      await store.updatePlanField('goal', 'new goal')

      expect(store.state.currentPlan?.goal).toBe('new goal')
      expect(mockUpdateAgent).toHaveBeenCalled()
    })
  })

  describe('sendMessage', () => {
    it('should add user message and streaming assistant message', async () => {
      mockCreateConversation.mockResolvedValue(mockConversation)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = mockAgent.specJson
      store.state.provider = 'ollama'
      store.state.defaultModel = 'dolphin-llama3:8b'

      mockOllamaChatStream.mockResolvedValue([])

      await store.sendMessage('Hello')

      expect(store.state.messages.length).toBe(2)
      expect(store.state.messages[0].role).toBe('user')
      expect(store.state.messages[0].content).toBe('Hello')
      expect(store.state.messages[1].role).toBe('assistant')
      expect(store.state.streaming).toBe(true)

      const [, model, , messages] = mockOllamaChatStream.mock.calls[0]
      expect(model).toBe('dolphin-llama3:8b')
      expect(messages.map(message => message.content)).toContain('Hello')
      expect(messages.some(message => message.role === 'assistant' && !message.content)).toBe(false)
    })

    it('should error if no agent selected', async () => {
      const store = useAgentStore()
      await store.sendMessage('Hello')
      expect(store.state.error).toBe('No agent selected')
    })
  })

  describe('renameAgent', () => {
    it('should rename an agent and update state', async () => {
      const renamed = { ...mockAgent, name: 'Renamed Agent' }
      mockRenameAgent.mockResolvedValue(renamed)
      mockListAgents.mockResolvedValue([mockAgentListItem])

      const store = useAgentStore()
      store.state.currentAgent = mockAgent

      const result = await store.renameAgent('test-agent-1', 'Renamed Agent')

      expect(mockRenameAgent).toHaveBeenCalledWith('test-agent-1', 'Renamed Agent')
      expect(store.state.currentAgent).toEqual(renamed)
      expect(result).toEqual(renamed)
    })
  })

  describe('duplicateAgent', () => {
    it('should duplicate an agent', async () => {
      const dup = { ...mockAgent, id: 'dup-agent-1', name: 'Copy of Test Agent' }
      mockDuplicateAgent.mockResolvedValue(dup)
      mockListAgents.mockResolvedValue([mockAgentListItem])

      const store = useAgentStore()
      const result = await store.duplicateAgent('test-agent-1', 'Copy of Test Agent')

      expect(mockDuplicateAgent).toHaveBeenCalledWith('test-agent-1', 'Copy of Test Agent')
      expect(result).toEqual(dup)
    })
  })

  describe('savePlanVersion', () => {
    it('should save a plan version and add to list', async () => {
      const version = { id: 'v1', agentId: 'test-agent-1', label: 'v1', specJson: { goal: 'v1 goal' }, createdAt: '2024-01-01' }
      mockSavePlanVersion.mockResolvedValue(version)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = { goal: 'v1 goal', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] }

      const result = await store.savePlanVersion('v1')

      expect(mockSavePlanVersion).toHaveBeenCalledWith({
        agentId: 'test-agent-1',
        label: 'v1',
        specJson: { goal: 'v1 goal', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] },
      })
      expect(store.state.planVersions).toContainEqual(version)
      expect(result).toEqual(version)
    })
  })

  describe('restorePlanVersion', () => {
    it('should restore a plan version', async () => {
      const plan = { goal: 'restored', steps: [], tools: [], knowledgeSources: [], triggers: [], outputs: [] }
      mockRestorePlanVersion.mockResolvedValue(plan)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent

      const result = await store.restorePlanVersion('v1')

      expect(mockRestorePlanVersion).toHaveBeenCalledWith('v1')
      expect(store.state.currentPlan).toEqual(plan)
      expect(result).toBe(true)
    })
  })

  describe('setProvider', () => {
    it('should set the provider', async () => {
      mockOllamaSetProvider.mockResolvedValue()

      const store = useAgentStore()
      await store.setProvider('openai')

      expect(mockOllamaSetProvider).toHaveBeenCalledWith('openai')
      expect(store.state.provider).toBe('openai')
    })
  })

  describe('setApiKey', () => {
    it('should set an API key', async () => {
      mockOllamaSetApiKey.mockResolvedValue()

      const store = useAgentStore()
      await store.setApiKey('openai', 'sk-test-key')

      expect(mockOllamaSetApiKey).toHaveBeenCalledWith('openai', 'sk-test-key')
      expect(store.state.apiKeys.openai).toBe('sk-test-key')
    })
  })
})
