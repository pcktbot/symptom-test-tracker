<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { getChatHistory, sendChatMessage, clearChatHistory, saveMemory } from '$lib/db';
  import { extractMemoryTag } from '$lib/utils';
  import type { ChatMessage } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let messages = $state<ChatMessage[]>([]);
  let streamingContent = $state('');
  let isStreaming = $state(false);
  let inputValue = $state('');
  let error = $state('');
  let messagesEl = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    const unlisteners: (() => void)[] = [];

    async function setup() {
      const history = await getChatHistory();
      messages = history;
      scrollToBottom();

      const u1 = await listen<string>('chat-token', (event) => {
        streamingContent += event.payload;
        scrollToBottom();
      });
      const u2 = await listen<void>('chat-done', async () => {
        if (streamingContent) {
          const { memory, displayText } = extractMemoryTag(streamingContent);
          if (memory) await saveMemory(memory);
          messages = [...messages, { role: 'assistant', content: displayText, created_at: new Date().toISOString() }];
          streamingContent = '';
        }
        isStreaming = false;
        scrollToBottom();
      });
      const u3 = await listen<string>('chat-error', (event) => {
        error = event.payload;
        isStreaming = false;
        streamingContent = '';
      });

      unlisteners.push(u1, u2, u3);
    }

    setup();
    return () => unlisteners.forEach(fn => fn());
  });

  function scrollToBottom() {
    setTimeout(() => {
      if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
    }, 10);
  }

  async function handleSend() {
    const text = inputValue.trim();
    if (!text || isStreaming) return;
    inputValue = '';
    error = '';
    isStreaming = true;
    messages = [...messages, { role: 'user', content: text, created_at: new Date().toISOString() }];
    scrollToBottom();
    try {
      await sendChatMessage(text);
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to send message';
      isStreaming = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  async function handleClear() {
    await clearChatHistory();
    messages = [];
    streamingContent = '';
  }
</script>

<div class="chat-panel">
  <div class="chat-header">
    <span class="chat-title">AI Assistant</span>
    <div class="chat-header-actions">
      {#if messages.length > 0}
        <button class="icon-btn" onclick={handleClear} title="Clear history">↺</button>
      {/if}
      <button class="icon-btn" onclick={onClose} title="Close">×</button>
    </div>
  </div>

  <div class="chat-messages" bind:this={messagesEl}>
    {#if messages.length === 0 && !isStreaming}
      <div class="empty-chat">
        <div class="empty-icon">✦</div>
        <p>Ask about your labs, symptoms, or any of your health data.</p>
      </div>
    {/if}

    {#each messages as msg (msg.created_at + msg.role)}
      <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
        {#if msg.role === 'assistant'}
          <div class="avatar"></div>
        {/if}
        <div class="bubble">{msg.content}</div>
      </div>
    {/each}

    {#if isStreaming}
      <div class="message assistant">
        <div class="avatar"></div>
        <div class="bubble streaming">
          {streamingContent}<span class="cursor"></span>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}
  </div>

  <div class="chat-input-area">
    <textarea
      bind:value={inputValue}
      onkeydown={handleKeydown}
      placeholder="Ask about your health data…"
      rows="2"
      disabled={isStreaming}
    ></textarea>
    <button class="send-btn" onclick={handleSend} disabled={isStreaming || !inputValue.trim()}>↑</button>
  </div>
</div>

<style>
  .chat-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .chat-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .chat-header-actions { display: flex; gap: 4px; }

  .icon-btn {
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 16px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    padding: 0;
  }
  .icon-btn:hover { background: var(--color-surface-raised); color: var(--color-text); }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty-chat {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    text-align: center;
    padding: 24px;
  }
  .empty-icon { font-size: 24px; margin-bottom: 10px; }
  .empty-chat p { font-size: 13px; line-height: 1.5; margin: 0; }

  .message { display: flex; gap: 8px; align-items: flex-start; }
  .message.user { flex-direction: row-reverse; }

  .avatar {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--color-accent), #cc88ff);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .bubble {
    max-width: 85%;
    padding: 8px 11px;
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .user .bubble {
    background: var(--color-accent);
    color: white;
    border-radius: 8px 8px 2px 8px;
  }

  .assistant .bubble {
    background: var(--color-surface-raised);
    color: var(--color-text);
    border-radius: 8px 8px 8px 2px;
  }

  .cursor {
    display: inline-block;
    width: 2px;
    height: 13px;
    background: var(--color-accent);
    margin-left: 2px;
    vertical-align: middle;
    animation: blink 1s step-end infinite;
  }
  @keyframes blink { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }

  .error-msg {
    background: #3a1a1a;
    color: #ff8888;
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 12px;
  }

  .chat-input-area {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    padding: 10px 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .chat-input-area textarea {
    flex: 1;
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 7px 10px;
    font-size: 13px;
    color: var(--color-text);
    resize: none;
    font-family: inherit;
    line-height: 1.4;
  }
  .chat-input-area textarea:focus { outline: none; border-color: var(--color-accent); }
  .chat-input-area textarea:disabled { opacity: 0.6; }

  .send-btn {
    width: 34px;
    height: 34px;
    border-radius: 6px;
    background: var(--color-accent);
    color: white;
    border: none;
    font-size: 16px;
    cursor: pointer;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
