import { useEffect } from 'react';
import { listen_focus_dom } from './event';

export function useFocusDom() {
  useEffect(() => {
    let unlisten: () => void;
    listen_focus_dom().then((fn) => {
      unlisten = fn;
    })
    return () => {
      if (unlisten) unlisten();
    }
  }, [])
}
