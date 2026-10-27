<script lang="ts">
  // Voice input (AR-1): dictation through the browser's speech recognition where it exists;
  // otherwise a recorded memo, attached as Instructions and transcribed during the build.
  import IconButton from '$lib/design/IconButton.svelte';
  import { toast } from '$lib/design/toasts.svelte';

  let { ondictate, onrecorded }: { ondictate: (text: string) => void; onrecorded: (f: File) => void } = $props();

  type Result = { isFinal: boolean; 0: { transcript: string } };
  interface Recognition {
    continuous: boolean;
    lang: string;
    onresult: ((e: { resultIndex: number; results: ArrayLike<Result> }) => void) | null;
    onend: (() => void) | null;
    start(): void;
    stop(): void;
  }
  type Speech = { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition };

  let stop = $state<(() => void) | null>(null);
  const done = () => (stop = null);

  async function start() {
    const Ctor = (window as unknown as Speech).SpeechRecognition ?? (window as unknown as Speech).webkitSpeechRecognition;
    if (Ctor) {
      const r = new Ctor();
      r.continuous = true;
      r.lang = navigator.language;
      r.onresult = (e) => {
        for (let i = e.resultIndex; i < e.results.length; i++) {
          const res = e.results[i];
          if (res?.isFinal) ondictate(res[0].transcript.trim());
        }
      };
      r.onend = done;
      r.start();
      stop = () => r.stop();
      return;
    }
    if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') {
      toast("Voice input isn't available in this browser.", 'error');
      return;
    }
    let stream: MediaStream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch {
      toast('Allow microphone access to record a voice memo.', 'error');
      return;
    }
    const rec = new MediaRecorder(stream);
    const chunks: Blob[] = [];
    rec.ondataavailable = (e) => chunks.push(e.data);
    rec.onstop = () => {
      stream.getTracks().forEach((t) => t.stop());
      done();
      const type = rec.mimeType || 'audio/webm';
      onrecorded(new File(chunks, `Voice memo ${new Date().toLocaleTimeString()}.webm`, { type }));
    };
    rec.start();
    stop = () => rec.stop();
  }
</script>

<span class="voice">
  {#if stop}<span class="live" aria-live="polite">Listening…</span>{/if}
  <IconButton
    icon={stop ? 'stop' : 'mic'}
    label={stop ? 'Stop listening' : 'Speak your goal'}
    variant="secondary"
    pressed={!!stop}
    onclick={() => (stop ? stop() : start())}
  />
</span>

<style>
  .voice { display: inline-flex; align-items: center; gap: 8px; }
  .live { font-size: 12px; font-weight: 600; color: var(--primary); }
</style>
