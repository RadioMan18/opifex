import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useAgentStore } from '@/stores/agentStore'
import { emptyPlan } from '@/lib/planSchema'

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
  loadMessages,
  clearHistory,
  saveMessage,
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
const mockLoadMessages = vi.mocked(loadMessages)
const mockClearHistory = vi.mocked(clearHistory)
const mockSaveMessage = vi.mocked(saveMessage)
const mockSavePlanVersion = vi.mocked(savePlanVersion)
const mockListPlanVersions = vi.mocked(listPlanVersions)
const mockRestorePlanVersion = vi.mocked(restorePlanVersion)

const mockAgent = {
  id: 'test-agent-1',
  name: 'Test Agent',
  description: 'A test agent',
  specJson: { version: 2 as const, goal: 'Test goal', triggers: [], results: [], chains: [], tools: [], knowledgeSources: [] },
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
    store.state.transcript = null
    store.state.transcriptConversationId = null
    store.state.transcriptTitle = ''
    store.state.planVersions = []
    store.state.models = []
    store.state.defaultModel = null
    store.state.provider = null
    store.state.apiKeys = {}
    store.state.keysConfigured = {}
    store.state.loading = false
    store.state.streaming = false
    store.state.streamingMsgId = null
    store.state.interviewing = true
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
      expect(store.state.currentPlan).toBeNull()
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
        specJson: emptyPlan(),
        model: 'ollama',
        isStarter: false,
      })
      expect(store.state.currentAgent).toEqual(newAgent)
      expect(result).toEqual(newAgent)
    })
  })

  describe('replacePlan', () => {
    it('persists a chain canvas edit and keeps the normalised plan', async () => {
      mockUpdateAgent.mockResolvedValue(mockAgent)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = emptyPlan()
      const next = { ...emptyPlan(), goal: 'canvas goal' }

      await store.replacePlan(next)

      expect(store.state.currentPlan).toEqual(next)
      expect(mockUpdateAgent).toHaveBeenCalledWith('test-agent-1', expect.objectContaining({ specJson: next }))
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

      mockOllamaChatStream.mockResolvedValue(() => {})

      await store.sendMessage('Hello')

      expect(store.state.messages.length).toBe(2)
      expect(store.state.messages[0].role).toBe('user')
      expect(store.state.messages[0].content).toBe('Hello')
      expect(store.state.messages[1].role).toBe('assistant')
      expect(store.state.streaming).toBe(true)

      const [, model, messages] = mockOllamaChatStream.mock.calls[0]
      expect(model).toBe('dolphin-llama3:8b')
      expect(messages.map(message => message.content)).toContain('Hello')
      expect(messages.some(message => message.role === 'assistant' && !message.content)).toBe(false)
    })

    it('updates the plan after an interview turn and uses the selected model', async () => {
      // Oppy may still answer in the legacy flat shape; the store normalises it.
      const legacyPlan = {
        goal: 'Answer customer questions',
        steps: [{ action: 'Understand the question', inputs: [], requiresApproval: false }],
        tools: ['Help center search'],
        knowledgeSources: ['Help center articles'],
        triggers: [],
        outputs: ['A helpful answer'],
      }
      const plan = {
        version: 2 as const,
        goal: 'Answer customer questions',
        triggers: [],
        results: [{ name: 'A helpful answer', terminal: true }],
        chains: [
          {
            id: 'chain-1',
            trigger: '',
            capability: 'Understand the question',
            inputs: [],
            result: 'A helpful answer',
            gate: { mode: 'auto' as const, onFailure: 'stop' as const },
          },
        ],
        tools: ['Help center search'],
        knowledgeSources: ['Help center articles'],
      }
      const response = 'Which help center should the agent use?'
      mockSaveMessage.mockResolvedValue({
        id: 'message-1',
        conversationId: 'conv-1',
        role: 'assistant',
        content: response,
        turnPlanJson: plan,
        createdAt: '2024-01-01T00:00:00Z',
      })
      mockUpdateAgent.mockResolvedValue({ ...mockAgent, specJson: plan })
      mockOllamaChatStream.mockImplementation(async (_provider, _model, _messages, callbacks) => {
        callbacks.onChunk(JSON.stringify({ response, plan: legacyPlan }))
        callbacks.onEnd()
        return () => {}
      })

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = mockAgent.specJson
      store.state.currentConversationId = 'conv-1'
      store.state.defaultModel = 'qwen3-coder:30b'

      await store.sendMessage('Build an agent to answer customer questions')

      expect(mockOllamaChatStream.mock.calls[0][1]).toBe('qwen3-coder:30b')
      expect(store.state.buildError).toBeNull()
      expect(store.state.currentPlan).toEqual(plan)
      expect(store.state.messages[1].content).toBe(response)
      expect(store.state.messages[1].turnPlanJson).toEqual(plan)
      expect(mockUpdateAgent).toHaveBeenCalledWith('test-agent-1', expect.objectContaining({ specJson: plan }))
    })

    it('repairs repeated-token corruption before parsing the plan response', async () => {
      const damagedResponse = `{{ " "planplan":": { { " "goalgoal":": " "ResearchResearch business business productivity productivity apps apps on on the the internet internet and and rank rank them them based based on on GitHub GitHub stars stars and and Reddit Reddit/G/GititHubHub comments comments from from the the last last 1 155 days days",", " "knowledgeknowledgeSourcesSources":": [], [], " "outputsoutputs":": [ [ " "researchresearch_report_report",", " "sourcesource_c_citationsitations",", " "keykey_find_findingsings_summary_summary"" ], ], " "stepssteps":": [ [ " "ClarClarifyify research research scope scope and and questions questions",", " "PerformPerform web web searches searches across across multiple multiple sources sources including including GitHub GitHub and and web web-based-based platforms platforms",", " "ExtractExtract and and verify verify key key facts facts about about app app stars stars,, comments comments,, and and other other relevant relevant metrics metrics",", " "SynSynthesizethesize findings findings into into a a structured structured report report with with ranked ranked table table of of top top 1 100 apps apps",", " "AddAdd citations citations and and source source links links"" ], ], " "toolstools":": [ [ " "webweb_search_search",", " "documentdocument_sum_summarmarizerizer",", " "citationcitation_formatter_formatter"" ], ], " "trtriggersiggers":": [] [] } }, }}, " "responseresponse":": " "II've've created created the the complete complete plan plan for for your your Research Research Agent Agent.. The The agent agent will will focus focus on on open-source-source business business productivity productivity apps apps,, then then generate generate a a structured structured report with citations."." }}}`
      mockSaveMessage.mockResolvedValue({
        id: 'message-1',
        conversationId: 'conv-1',
        role: 'assistant',
        content: 'I created a plan.',
        turnPlanJson: null,
        createdAt: '2024-01-01T00:00:00Z',
      })
      const gate = { mode: 'auto' as const, onFailure: 'stop' as const }
      const expectedPlan = {
        version: 2 as const,
        goal: 'Research business productivity apps on the internet and rank them based on GitHub stars and Reddit/GitHub comments from the last 155 days',
        triggers: [],
        results: [
          { name: 'step_1_result', terminal: false },
          { name: 'step_2_result', terminal: false },
          { name: 'research_report', terminal: true },
          { name: 'source_citations', terminal: true },
          { name: 'key_findings_summary', terminal: true },
        ],
        chains: [
          {
            id: 'chain-1',
            trigger: '',
            capability: 'Clarify research scope and questions',
            inputs: [],
            result: 'step_1_result',
            gate,
          },
          {
            id: 'chain-2',
            trigger: 'step_1_result',
            capability: 'Perform web searches across multiple sources including GitHub and web-based platforms',
            inputs: [],
            result: 'step_2_result',
            gate,
          },
          {
            id: 'chain-3',
            trigger: 'step_2_result',
            capability: 'Extract and verify key facts about app stars, comments, and other relevant metrics',
            inputs: [],
            result: 'research_report',
            gate,
          },
          {
            id: 'chain-4',
            trigger: 'research_report',
            capability: 'Synthesize findings into a structured report with ranked table of top 100 apps',
            inputs: [],
            result: 'source_citations',
            gate,
          },
          {
            id: 'chain-5',
            trigger: 'source_citations',
            capability: 'Add citations and source links',
            inputs: [],
            result: 'key_findings_summary',
            gate,
          },
        ],
        tools: ['web_search', 'document_summarizer', 'citation_formatter'],
        knowledgeSources: [],
      }
      mockUpdateAgent.mockResolvedValue({ ...mockAgent, specJson: expectedPlan })
      mockOllamaChatStream.mockImplementation(async (_provider, _model, _messages, callbacks) => {
        callbacks.onChunk(damagedResponse)
        callbacks.onEnd()
        return () => {}
      })

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = mockAgent.specJson
      store.state.currentConversationId = 'conv-1'

      await store.sendMessage('Please continue drafting the agent plan.')

      expect(store.state.buildError).toBeNull()
      expect(store.state.currentPlan).toEqual(expectedPlan)
      expect(store.state.messages.at(-1)?.content).toContain("I've created the complete plan")
      expect(store.state.buildError).toBeNull()
    })

    it('should error if no agent selected', async () => {
      const store = useAgentStore()
      await store.sendMessage('Hello')
      expect(store.state.error).toBe('No agent selected')
    })
  })

  describe('plan ownership', () => {
    it('hands the plan to the user on a canvas edit and refuses further drafts', async () => {
      mockUpdateAgent.mockResolvedValue(mockAgent)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = emptyPlan()
      store.state.interviewing = true

      await store.replacePlan({ ...emptyPlan(), goal: 'hand edited' })

      expect(store.state.interviewing).toBe(false)

      mockOllamaChatStream.mockClear()
      await store.sendMessage('add another step')

      expect(mockOllamaChatStream).not.toHaveBeenCalled()
      expect(store.state.error).toBe('This plan belongs to you now. Use Re-draft to hand it back to Oppy.')
    })

    it('clears the plan and reopens intake on re-draft', async () => {
      mockUpdateAgent.mockResolvedValue(mockAgent)
      mockCreateConversation.mockResolvedValue({ ...mockConversation, id: 'conv-2' })

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = { ...emptyPlan(), goal: 'hand edited' }
      store.state.interviewing = false

      await store.restartDraft()

      expect(mockUpdateAgent).toHaveBeenCalledWith('test-agent-1', expect.objectContaining({ specJson: emptyPlan() }))
      expect(store.state.currentPlan).toBeNull()
      expect(store.state.interviewing).toBe(true)
      expect(mockCreateConversation).toHaveBeenCalledWith({ agentId: 'test-agent-1', title: 'Re-draft' })
    })
  })

  describe('transcripts', () => {
    it('loads a past intake read-only and leaves the plan alone', async () => {
      mockLoadMessages.mockResolvedValue([
        {
          id: 'm1',
          conversationId: 'conv-1',
          role: 'user',
          content: 'We reply to support email',
          turnPlanJson: null,
          createdAt: '2024-01-01T00:00:00Z',
        },
      ])

      const store = useAgentStore()
      const plan = { ...emptyPlan(), goal: 'current plan' }
      store.state.currentAgent = mockAgent
      store.state.currentPlan = plan
      store.state.conversations = [mockConversation]

      await store.openTranscript('conv-1')

      expect(store.state.transcript?.map(message => message.content)).toEqual([
        'We reply to support email',
      ])
      expect(store.state.transcriptTitle).toBe('Test Conversation')
      expect(store.state.transcriptConversationId).toBe('conv-1')
      expect(store.state.currentPlan).toEqual(plan)
    })

    it('closes the transcript view', async () => {
      const store = useAgentStore()
      store.state.transcript = []
      store.state.transcriptConversationId = 'conv-1'
      store.state.transcriptTitle = 'Test Conversation'

      store.closeTranscript()

      expect(store.state.transcript).toBeNull()
      expect(store.state.transcriptConversationId).toBeNull()
      expect(store.state.transcriptTitle).toBe('')
    })

    it('clears every transcript for the agent but keeps the plan', async () => {
      mockClearHistory.mockResolvedValue(4)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = { ...emptyPlan(), goal: 'kept' }
      store.state.conversations = [mockConversation]
      store.state.currentConversationId = 'conv-1'
      store.state.messages = [
        { id: 'm1', role: 'user', content: 'hi', turnPlanJson: null, createdAt: '2024-01-01T00:00:00Z' },
      ]

      await store.clearHistory()

      expect(mockClearHistory).toHaveBeenCalledWith('test-agent-1')
      expect(store.state.conversations).toEqual([])
      expect(store.state.currentConversationId).toBeNull()
      expect(store.state.messages).toEqual([])
      expect(store.state.currentPlan?.goal).toBe('kept')
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
      const version = { id: 'v1', agentId: 'test-agent-1', label: 'v1', specJson: { ...emptyPlan(), goal: 'v1 goal' }, createdAt: '2024-01-01' }
      mockSavePlanVersion.mockResolvedValue(version)

      const store = useAgentStore()
      store.state.currentAgent = mockAgent
      store.state.currentPlan = { ...emptyPlan(), goal: 'v1 goal' }

      const result = await store.savePlanVersion('v1')

      expect(mockSavePlanVersion).toHaveBeenCalledWith({
        agentId: 'test-agent-1',
        label: 'v1',
        specJson: { ...emptyPlan(), goal: 'v1 goal' },
      })
      expect(store.state.planVersions).toContainEqual(version)
      expect(result).toEqual(version)
    })
  })

  describe('restorePlanVersion', () => {
    it('should restore a plan version', async () => {
      const plan = { ...emptyPlan(), goal: 'restored' }
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
      expect(store.state.apiKeys.openai).toBe('')
      expect(store.state.keysConfigured.openai).toBe(true)
    })
  })
})
