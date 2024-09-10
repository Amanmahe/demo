import { invoke } from "@tauri-apps/api/tauri";

async function handleGetPorts(setPorts: any) {
  const ports = await invoke("get_ports", {});
  setPorts(ports);
}

async function handleConnect(port: string, baud: string, ending: string, setIsConnected: any) {
  invoke("set_port_items", {port, baud, ending});
  const isConnected = await invoke("handle_serial_connect", {});
  setIsConnected(isConnected);
}

function getBaudList() { 
  return [
    "57600",
    "74880",
    "115200",
    "230400",
  ];
}




async function sendError(input: String) {
  await invoke("emit_error", {input})
}

export { handleGetPorts, handleConnect, getBaudList, sendError } 