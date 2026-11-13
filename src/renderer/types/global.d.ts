import { NovaBridge } from '../../preload/preload';

declare global {
  interface Window {
    nova: NovaBridge;
  }
}

