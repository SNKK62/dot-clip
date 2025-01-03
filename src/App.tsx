import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isRegistered, register } from "@tauri-apps/plugin-global-shortcut";
import { LogicalPosition, getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

function App() {
  const [data, setData] = useState<string>("");

  async function startWatchClipboard() {
    invoke("watch_clipboard");
  }

  async function getPreviousClipboard() {
    const clipboard: string = await invoke("get_previous_content");
    setData(clipboard);
  }

  useEffect(()=> {
    startWatchClipboard()

    const setup = async () => {
      const isRegist = await isRegistered("Control+Shift+C");
      console.log(isRegist);
			if (isRegist) return;
			await register("Control+Shift+C", async (event) => {
        if (event.state !== "Pressed") return;
        const isRegist = await isRegistered("Control+Shift+C");
        console.log(isRegist);
        const appWindow = getCurrentWindow();
        getPreviousClipboard();
				const cursorPosition = (await invoke("get_cursor_position")) as {
					x: number;
					y: number;
				};
				await appWindow.setPosition(
					new LogicalPosition(cursorPosition.x, cursorPosition.y),
				);
        await appWindow.show();
        await appWindow.setFocus();
        await appWindow.setShadow(true);
        await appWindow.setAlwaysOnTop(true);
        await appWindow.setMaximizable(false);
        await appWindow.setMinimizable(false);
        await appWindow.setClosable(false);
        await appWindow.setResizable(false);
			});
		};
    setup();
  }, [])

  return (
    <main className="container">
      <ul>
        <li>{data}</li>
      </ul>
    </main>
  );
}

export default App;
