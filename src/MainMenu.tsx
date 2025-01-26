import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isRegistered, register } from "@tauri-apps/plugin-global-shortcut";
import { LogicalPosition, getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { useFocusDom } from "./hooks";

export function MainMenu() {
  const [data, setData] = useState<string[]>([]);
  useFocusDom();

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
        invoke("open_submenu");
        invoke("focus_webview_window", {name: "main_menu"});
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
    const handleKeyDown = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "Escape") {
        invoke("close_all");
      } else if (e.key === "ArrowRight") {
        invoke("focus_webview_window", {name: "sub_menu"});
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
          return (<li key={i}><button>{el}</button></li>)
        })}
      </ul>
    </main>
  );
}

