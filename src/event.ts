import { getCurrentWindow } from "@tauri-apps/api/window";

export const listen_focus_dom = async (): Promise<() => void> => {
  const appWindow = getCurrentWindow();
  const unlisten = await appWindow.listen("focus-dom", ({event, payload}) => {
    console.log("event: ", event);
    console.log("focus: ", payload);
    const buttonElement = document.querySelector('button');
    if (buttonElement) {
      buttonElement.focus();
    }
  })
  return unlisten;
}
