export class AudioParam {
  __engine_node: AudioNode;
  __engine_name: string;

  constructor(node: AudioNode, name: string) {
    this.__engine_node = node;
    this.__engine_name = String(name);
  }


  get value(): number {
    return __engine_audio_param_get(this.__engine_node.__engine_audio_id, this.__engine_name);
  }


  set value(value: number) {
    __engine_audio_param_set(this.__engine_node.__engine_audio_id, this.__engine_name, value);
  }


  setValueAtTime(value: number, startTime: number): AudioParam {
    __engine_audio_param_set_at_time(this.__engine_node.__engine_audio_id, this.__engine_name, value, startTime);
    return this;
  }


  linearRampToValueAtTime(value: number, endTime: number): AudioParam {
    __engine_audio_param_linear_ramp_to_value_at_time(this.__engine_node.__engine_audio_id, this.__engine_name, value, endTime);
    return this;
  }


  exponentialRampToValueAtTime(value: number, endTime: number): AudioParam {
    __engine_audio_param_exponential_ramp_to_value_at_time(this.__engine_node.__engine_audio_id, this.__engine_name, value, endTime);
    return this;
  }
}

export class AudioNode {
  context: AudioContext;
  __engine_audio_id: number;
  __engine_audio_kind: string;

  constructor(context: AudioContext, id: number, kind: string) {
    this.context = context;
    this.__engine_audio_id = id;
    this.__engine_audio_kind = kind;
  }

  connect(destination: AudioNode): AudioNode {
    if (!destination || destination.__engine_audio_id == null) {
      throw new TypeError("AudioNode.connect() requires an AudioNode destination");
    }
    __engine_audio_connect(this.context.__engine_audio_context_id, this.__engine_audio_id, destination.__engine_audio_id);
    return destination;
  }

  disconnect(): void {
    __engine_audio_disconnect(this.__engine_audio_id);
  }
}

export class AudioDestinationNode extends AudioNode {
  constructor(context: AudioContext) {
    super(context, 0, "destination");
  }
}

export class GainNode extends AudioNode {
  gain: AudioParam;

  constructor(context: AudioContext, id: number) {
    super(context, id, "gain");
    this.gain = new AudioParam(this, "gain");
  }
}

export class OscillatorNode extends AudioNode {
  frequency: AudioParam;
  detune: AudioParam;

  constructor(context: AudioContext, id: number) {
    super(context, id, "oscillator");
    this.frequency = new AudioParam(this, "frequency");
    this.detune = new AudioParam(this, "detune");
  }

  get type(): EngineOscillatorType | string {
    return __engine_audio_oscillator_get_type(this.__engine_audio_id);
  }

  set type(value: EngineOscillatorType | string) {
    __engine_audio_oscillator_set_type(this.__engine_audio_id, value);
  }

  start(when: number = 0): void {
    __engine_audio_source_start(this.__engine_audio_id, when);
  }

  stop(when: number = 0): void {
    __engine_audio_source_stop(this.__engine_audio_id, when);
  }
}

export class BiquadFilterNode extends AudioNode {
  frequency: AudioParam;
  detune: AudioParam;
  Q: AudioParam;
  gain: AudioParam;

  constructor(context: AudioContext, id: number) {
    super(context, id, "biquadFilter");

    this.frequency = new AudioParam(this, "frequency");
    this.detune = new AudioParam(this, "detune");
    this.Q = new AudioParam(this, "Q");
    this.gain = new AudioParam(this, "gain");
  }

  get type(): EngineBiquadFilterType | string {
    return __engine_audio_biquad_get_type(this.__engine_audio_id);
  }

  set type(value: EngineBiquadFilterType | string) {
    __engine_audio_biquad_set_type(this.__engine_audio_id, value);
  }
}

export class AudioBuffer {
  __engine_context_id: number;
  __engine_audio_buffer_id: number | null;
  __engine_channels: number[][] | null;
  length: number;
  sampleRate: number;
  numberOfChannels: number;
  duration: number;

  constructor(context: AudioContext, id: number | null, channels: number[][] | null, length: number, sampleRate: number) {
    this.__engine_context_id = context.__engine_audio_context_id;
    this.__engine_audio_buffer_id = id;
    this.__engine_channels = channels || null;

    this.length = Math.max(0, Math.floor(Number(length || 0)));
    this.sampleRate = Number(sampleRate || context.sampleRate);
    this.numberOfChannels = channels ? channels.length : __engine_audio_buffer_number_of_channels(id);
    this.duration = this.sampleRate > 0 ? this.length / this.sampleRate : 0;
  }


  getChannelData(channel: number): number[] {
    if (!this.__engine_channels && this.__engine_audio_buffer_id != null) {
      this.__engine_channels = __engine_audio_buffer_to_channels(this.__engine_audio_buffer_id);
    }

    const index = Math.max(0, Math.floor(Number(channel || 0)));
    const channels = this.__engine_channels;

    if (!channels || !channels[index]) {
      throw new RangeError("AudioBuffer channel index out of range");
    }

    return channels[index];
  }

  __engine_materialize(): number {
    if (this.__engine_audio_buffer_id == null) {
      this.__engine_audio_buffer_id = __engine_audio_create_buffer_from_channels(
        this.__engine_context_id,
        this.__engine_channels,
        this.sampleRate
      );
    }

    return this.__engine_audio_buffer_id;
  }
}

export class AudioBufferSourceNode extends AudioNode {
  playbackRate: AudioParam;
  detune: AudioParam;
  __buffer: AudioBuffer | null;

  constructor(context: AudioContext, id: number) {
    super(context, id, "bufferSource");

    this.playbackRate = new AudioParam(this, "playbackRate");
    this.detune = new AudioParam(this, "detune");
    this.__buffer = null;
  }

  get buffer(): AudioBuffer | null {
    return this.__buffer;
  }

  set buffer(value: AudioBuffer) {
    if (!(value instanceof AudioBuffer)) {
      throw new TypeError("AudioBufferSourceNode.buffer must be an AudioBuffer");
    }

    this.__buffer = value;
    __engine_audio_buffer_source_set_buffer(this.__engine_audio_id, value.__engine_materialize());
  }

  start(when: number = 0, offset: number = 0, duration: number | undefined = undefined): void {
    if (duration === undefined) {
      __engine_audio_source_start_at_with_offset(this.__engine_audio_id, when, offset);
    } else {
      __engine_audio_source_start_at_with_offset_and_duration(this.__engine_audio_id, when, offset, duration);
    }
  }

  stop(when: number = 0): void {
    __engine_audio_source_stop(this.__engine_audio_id, when);
  }
}

export class AudioContext {
  __engine_audio_context_id: number;
  readonly destination: AudioDestinationNode;

  constructor(options: EngineAudioContextOptions = {}) {

    this.__engine_audio_context_id = __engine_audio_context_create(options.sampleRate);

    this.destination = new AudioDestinationNode(this);
  }

  get currentTime(): number {
    return __engine_audio_context_current_time(this.__engine_audio_context_id);
  }

  get sampleRate(): number {
    return __engine_audio_context_sample_rate(this.__engine_audio_context_id);
  }

  get state(): EngineAudioContextState | string {
    return __engine_audio_context_state(this.__engine_audio_context_id);
  }

  resume(): Promise<void> {
    __engine_audio_context_resume(this.__engine_audio_context_id);
    return Promise.resolve();
  }

  suspend(): Promise<void> {
    __engine_audio_context_suspend(this.__engine_audio_context_id);
    return Promise.resolve();
  }

  close(): Promise<void> {
    __engine_audio_context_close(this.__engine_audio_context_id);
    return Promise.resolve();
  }

  createGain(): GainNode {
    return new GainNode(this, __engine_audio_create_gain(this.__engine_audio_context_id));
  }

  createBufferSource(): AudioBufferSourceNode {
    return new AudioBufferSourceNode(this, __engine_audio_create_buffer_source(this.__engine_audio_context_id));
  }

  createOscillator(): OscillatorNode {
    return new OscillatorNode(this, __engine_audio_create_oscillator(this.__engine_audio_context_id));
  }

  createBiquadFilter(): BiquadFilterNode {
    return new BiquadFilterNode(this, __engine_audio_create_biquad_filter(this.__engine_audio_context_id));
  }

  createBuffer(numberOfChannels: number, length: number, sampleRate: number = this.sampleRate): AudioBuffer {
    const channels: number[][] = [];
    const channelCount = Math.max(1, Math.floor(Number(numberOfChannels || 1)));
    const frameCount = Math.max(0, Math.floor(Number(length || 0)));
    for (let i = 0; i < channelCount; i++) {
      channels.push(new Array(frameCount).fill(0));
    }
    return new AudioBuffer(this, null, channels, frameCount, sampleRate);
  }

  decodeAudioData(
    data: string | ArrayBuffer | ArrayLike<number> | object,
    successCallback?: EngineDecodeSuccessCallback,
    errorCallback?: EngineDecodeErrorCallback,
  ): Promise<AudioBuffer> {
    let promise: Promise<AudioBuffer>;
    try {
      const id = __engine_audio_decode_audio_data(this.__engine_audio_context_id, data);
      const info = __engine_audio_buffer_info(id);
      promise = Promise.resolve(new AudioBuffer(this, id, null, info.length, info.sampleRate));
    } catch (error) {
      promise = Promise.reject(error);
    }
    if (typeof successCallback === "function") {
      promise.then(successCallback);
    }
    if (typeof errorCallback === "function") {
      promise.catch(errorCallback);
    }
    return promise;
  }
}
