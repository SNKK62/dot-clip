import { useEffect, useState } from "react";
import { LogicalPosition, getCurrentWindow, getAllWindows } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

export function SubMenu() {
  useEffect(() => {
    const appWindow = getCurrentWindow();
    const handleKeyDown = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "ArrowLeft") {
        // appWindow.hide();
        getAllWindows().then(windows => {
          const mainWindow = windows.find(w => w.label === "main_menu");
          if (mainWindow) {
            mainWindow.setFocus();
          }
        })
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

