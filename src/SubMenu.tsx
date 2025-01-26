import { useEffect, useState } from "react";
import { LogicalPosition, getCurrentWindow, getAllWindows } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { useFocusDom } from "./hooks";

export function SubMenu() {
  useFocusDom();
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "ArrowLeft") {
        // appWindow.hide();
        invoke("focus_webview_window", {name: "main_menu"});
      } else if (e.key === "Enter") {
        invoke("close_and_paste");
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    }
  }, [])
  return (
    <main>
      <button>This is submenu</button>
    </main>
  );
}

