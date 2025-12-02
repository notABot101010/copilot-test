import { Signal, signal } from '@preact/signals';

export interface SignalingMessage {
  type: 'offer' | 'answer' | 'ice-candidate' | 'peer-joined' | 'peer-left' | 'room-full' | 'waiting';
  sdp?: string;
  candidate?: string;
}

export interface WebRTCService {
  connectionState: Signal<RTCPeerConnectionState>;
  iceConnectionState: Signal<RTCIceConnectionState>;
  localStream: Signal<MediaStream | null>;
  remoteStream: Signal<MediaStream | null>;
  isInitiator: Signal<boolean>;
  peerJoined: Signal<boolean>;
  roomFull: Signal<boolean>;
  error: Signal<string | null>;
  connect(roomId: string, isInitiator: boolean): Promise<void>;
  disconnect(): void;
  startCall(): Promise<void>;
  answerCall(): Promise<void>;
}

export function createWebRTCService(): WebRTCService {
  const connectionState = signal<RTCPeerConnectionState>('new');
  const iceConnectionState = signal<RTCIceConnectionState>('new');
  const localStream = signal<MediaStream | null>(null);
  const remoteStream = signal<MediaStream | null>(null);
  const isInitiator = signal<boolean>(false);
  const peerJoined = signal<boolean>(false);
  const roomFull = signal<boolean>(false);
  const error = signal<string | null>(null);

  let peerConnection: RTCPeerConnection | null = null;
  let websocket: WebSocket | null = null;
  let currentRoomId: string | null = null;

  const iceServers: RTCIceServer[] = [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ];

  async function getLocalStream(): Promise<MediaStream> {
    if (localStream.value) {
      return localStream.value;
    }
    
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: true,
        audio: true,
      });
      localStream.value = stream;
      return stream;
    } catch (err) {
      error.value = 'Failed to access camera/microphone';
      throw err;
    }
  }

  function createPeerConnection(): RTCPeerConnection {
    const pc = new RTCPeerConnection({ iceServers });

    pc.onconnectionstatechange = () => {
      connectionState.value = pc.connectionState;
    };

    pc.oniceconnectionstatechange = () => {
      iceConnectionState.value = pc.iceConnectionState;
    };

    pc.onicecandidate = (event) => {
      if (event.candidate && websocket) {
        const message: SignalingMessage = {
          type: 'ice-candidate',
          candidate: JSON.stringify(event.candidate),
        };
        websocket.send(JSON.stringify(message));
      }
    };

    pc.ontrack = (event) => {
      if (event.streams[0]) {
        remoteStream.value = event.streams[0];
      }
    };

    return pc;
  }

  async function handleSignalingMessage(message: SignalingMessage) {
    switch (message.type) {
      case 'waiting':
        // We're the initiator, waiting for peer
        isInitiator.value = true;
        break;

      case 'peer-joined':
        peerJoined.value = true;
        // If we're initiator, start the call when peer joins
        if (isInitiator.value) {
          await startCall();
        }
        break;

      case 'peer-left':
        peerJoined.value = false;
        remoteStream.value = null;
        connectionState.value = 'disconnected';
        break;

      case 'room-full':
        roomFull.value = true;
        error.value = 'Room is full';
        break;

      case 'offer':
        if (!peerConnection) {
          peerConnection = createPeerConnection();
          const stream = await getLocalStream();
          stream.getTracks().forEach(track => {
            peerConnection!.addTrack(track, stream);
          });
        }
        
        if (message.sdp) {
          await peerConnection.setRemoteDescription({
            type: 'offer',
            sdp: message.sdp,
          });
          await answerCall();
        }
        break;

      case 'answer':
        if (peerConnection && message.sdp) {
          await peerConnection.setRemoteDescription({
            type: 'answer',
            sdp: message.sdp,
          });
        }
        break;

      case 'ice-candidate':
        if (peerConnection && message.candidate) {
          const candidate = JSON.parse(message.candidate);
          await peerConnection.addIceCandidate(candidate);
        }
        break;
    }
  }

  async function connect(roomId: string, initiator: boolean): Promise<void> {
    currentRoomId = roomId;
    isInitiator.value = initiator;

    // Get local stream first
    await getLocalStream();

    // Connect to signaling server
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const wsUrl = `${protocol}//${host}/ws/${roomId}`;

    websocket = new WebSocket(wsUrl);

    websocket.onopen = () => {
      console.log('Connected to signaling server');
    };

    websocket.onmessage = async (event) => {
      try {
        const message = JSON.parse(event.data) as SignalingMessage;
        await handleSignalingMessage(message);
      } catch (err) {
        console.error('Failed to handle signaling message:', err);
      }
    };

    websocket.onerror = (event) => {
      console.error('WebSocket error:', event);
      error.value = 'Connection error';
    };

    websocket.onclose = () => {
      console.log('Disconnected from signaling server');
    };
  }

  async function startCall(): Promise<void> {
    if (!websocket) {
      error.value = 'Not connected to signaling server';
      return;
    }

    peerConnection = createPeerConnection();
    
    const stream = await getLocalStream();
    stream.getTracks().forEach(track => {
      peerConnection!.addTrack(track, stream);
    });

    const offer = await peerConnection.createOffer();
    await peerConnection.setLocalDescription(offer);

    const message: SignalingMessage = {
      type: 'offer',
      sdp: offer.sdp,
    };
    websocket.send(JSON.stringify(message));
  }

  async function answerCall(): Promise<void> {
    if (!websocket || !peerConnection) {
      error.value = 'Not ready to answer call';
      return;
    }

    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);

    const message: SignalingMessage = {
      type: 'answer',
      sdp: answer.sdp,
    };
    websocket.send(JSON.stringify(message));
  }

  function disconnect(): void {
    if (localStream.value) {
      localStream.value.getTracks().forEach(track => track.stop());
      localStream.value = null;
    }

    if (peerConnection) {
      peerConnection.close();
      peerConnection = null;
    }

    if (websocket) {
      websocket.close();
      websocket = null;
    }

    remoteStream.value = null;
    connectionState.value = 'new';
    iceConnectionState.value = 'new';
    peerJoined.value = false;
    roomFull.value = false;
    currentRoomId = null;
  }

  return {
    connectionState,
    iceConnectionState,
    localStream,
    remoteStream,
    isInitiator,
    peerJoined,
    roomFull,
    error,
    connect,
    disconnect,
    startCall,
    answerCall,
  };
}
