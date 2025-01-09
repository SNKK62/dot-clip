import { useEffect, useState } from "react";
import { MainMenu } from "./MainMenu";
import { SubMenu } from "./SubMenu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

function App() {
  const [label, setLabel] = useState<string | undefined>(undefined);

  useEffect(() => {
      const appWindow = getCurrentWindow();
      const label = appWindow.label;
    setLabel(label);
    console.log("label:", label);
  }, []);

  if (!label) return null;

  if (label === "main_menu") {
    return <MainMenu />;
  } else if (label === "sub_menu") {
    return <SubMenu />;
  } else {
    throw new Error(`Unknown window label: ${label}`);
  }
}

export default App;
