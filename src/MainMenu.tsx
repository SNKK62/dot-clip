import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isRegistered, register } from "@tauri-apps/plugin-global-shortcut";
import { LogicalPosition, getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

export function MainMenu() {
  const [data, setData] = useState<string[]>([]);

  async function startWatchClipboard() {
    invoke("watch_clipboard");
  }

  async function getPreviousClipboard() {
    const clipboard: string[] = await invoke("get_previous_content");
    setData(clipboard);
  }

  useEffect(()=> {
    startWatchClipboard()

    const setup = async () => {
      const hotkey = "Control+Shift+C";
      const alreadyRegistered = await isRegistered(hotkey);
      console.log("is already registered:", alreadyRegistered);
      if (alreadyRegistered) return;
      await register(hotkey, async (event) => {
        if (event.state !== "Pressed") return; // skip the release event
        const appWindow = getCurrentWindow();
        if (await appWindow.isVisible()) {
          return
        }
        getPreviousClipboard();
        // set the window position to the cursor position
        const cursorPosition = (await invoke("get_cursor_position")) as {
          x: number;
          y: number;
        };
        await appWindow.setPosition(
          new LogicalPosition(cursorPosition.x, cursorPosition.y),
        );
        await appWindow.show();
        await invoke("open_submenu");
        await appWindow.setFocus();
        // await appWindow.setShadow(true);
        // await appWindow.setAlwaysOnTop(true);
        // await appWindow.setMaximizable(false);
        // await appWindow.setMinimizable(false);
        // await appWindow.setClosable(false);
        // await appWindow.setResizable(false);
      });
    };
    setup();
  }, [])

  useEffect(() => {
    const appWindow = getCurrentWindow();
    const handleKeyDown = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "Escape") {
        appWindow.hide();
        invoke("close_submenu");
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    }
  }, [])

  return (
    <main>
      <ul>
        {data.map((el, i) => {
          return (<li key={i}>{el}</li>)
        })}
      </ul>
    </main>
  );
}

