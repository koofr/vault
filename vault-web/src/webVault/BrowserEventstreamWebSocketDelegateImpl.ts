import { BrowserEventstreamWebSocketDelegate } from '../vault-wasm/vault-wasm';

export class BrowserEventstreamWebSocketDelegateImpl
  implements BrowserEventstreamWebSocketDelegate
{
  ws?: WebSocket;
  onOpen?: () => void;
  onMessage?: (data: string) => void;
  onClose?: () => void;

  open = (
    url: string,
    onOpen: () => void,
    onMessage: (data: string) => void,
    onClose: () => void,
  ): void => {
    this.onOpen = onOpen;
    this.onMessage = onMessage;
    this.onClose = onClose;

    // open /#noeventstream to test the app without event stream
    if (/noeventstream/.test(document.location.hash)) {
      return;
    }

    this.ws = new WebSocket(url);

    this.ws.addEventListener('open', () => {
      if (this.onOpen !== undefined) {
        this.onOpen();
      }
    });

    this.ws.addEventListener('message', (event) => {
      if (this.onMessage !== undefined) {
        this.onMessage(event.data);
      }
    });

    this.ws.addEventListener('close', () => {
      if (this.onClose !== undefined) {
        this.onClose();
      }
    });

    window.addEventListener('offline', this.onOffline);
  };

  send = (data: string): void => {
    this.ws?.send(data);
  };

  close = (): void => {
    this.onOpen = undefined;
    this.onMessage = undefined;
    this.onClose = undefined;

    window.removeEventListener('offline', this.onOffline);

    this.ws?.close();

    this.ws = undefined;
  };

  onOffline = () => {
    if (this.ws?.readyState === WebSocket.OPEN && this.onClose !== undefined) {
      console.log('Eventstream closing due to network offline');

      window.removeEventListener('offline', this.onOffline);

      this.onClose();
    }
  };
}
