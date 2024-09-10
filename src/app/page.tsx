"use client";
import { invoke } from "@tauri-apps/api/tauri";
import { emit, listen } from "@tauri-apps/api/event";
import { handleGetPorts, getBaudList, handleConnect, sendError } from "../utils/serial"
import Connection from "../components/Connection";
import Navbar from "../components/Navbar";
import React, { useState,useEffect } from "react";
import Canvas from "../components/Canvas";

export type BitSelection = "ten" | "twelve" | "fourteen" | "auto";

const   Page = () => {
  const [data, setData] = useState(""); // Data from the serial port
  const [selectedBits, setSelectedBits] = useState<BitSelection>("auto"); // Selected bits
  const [isConnected, setIsConnected] = useState<boolean>(false); // Connection status
  const [isGridView, setIsGridView] = useState<boolean>(true); // Grid view state
  const [isDisplay, setIsDisplay] = useState<boolean>(true); // Display state
  type Payload = {
    message: Uint8Array;
};
  async function startSerialEventListener() {
    await listen<Payload>("updateSerial", (event: any) => {
        console.log(event.payload.counter);
        console.log(event.payload.data);
        //what is that we want to mak is cominng from did not coming from serial port e the the data which
    });
}
useEffect(() => {
        startSerialEventListener();
    }, []);

//port
function MenuItem({ text, onClick}: any) {
  return <li onClick={onClick} className="px-6 py-2 bg-white hover:bg-zinc-400 cursor-pointer text-black">{text}</li>;
}

function SubMenu({ text, setHook, menuItemList }: any) {
  const [isSubOpen, setIsSubOpen] = useState(false);

  const openDropdown = () => {
    setIsSubOpen(true);
  };

  const closeDropdown = () => {
    setIsSubOpen(false);
  };

  function handleSelection (item: string){
    setHook(item)
  }
  
  return (
    <li
      className="relative"
      onMouseEnter={openDropdown}
      onMouseLeave={closeDropdown}
    >
      <li className="px-6 py-2 cursor-pointer text-black bg-white hover:bg-zinc-400">{text}</li>
      {isSubOpen && (
        <ul className="absolute w-max bg-white left-full top-0">
          {menuItemList.map((item: any, index: any) => (
              <MenuItem key={index} text={item} onClick={() => handleSelection(item)}/>
          ))}
        </ul>
      )}
    </li>
  );
}

function Serial() {
  const [baud, setBaud] = useState("57600");
  const [port, setPort] = useState("None");
  const [portList, setPortList] = useState(["None"]);
  const [ending, setEnding] = useState("None");
  const [isConnected, setIsConnected] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  // open dropdown and also gets dynamic data
  function openDropdown() {
    setIsDropdownOpen(true);
    handleGetPorts(setPortList);
  };

  function closeDropdown() {
    setIsDropdownOpen(false);
  };

  // same as payload
  type Payload = {
    connected: string;
  };

  async function startSerialEventListenerOnIsConnection() {
    await listen<Payload>("isConnected", (event: any) => {
      console.log(event.payload.message);
      if (event.payload.message === "disconnected") { 
        setIsConnected(false);
      }
      sendError("Port has been unexpectedly disconected");
    });
  } 

  

  useEffect(() => {
    startSerialEventListenerOnIsConnection();
  }, []);

  return (
    <nav
      onMouseEnter={openDropdown}
      onMouseLeave={closeDropdown}
    >
      <ul className="flex justify-center items-center py-2 cursor-pointer">
        <li className="relative h-fit px-4">
          <span>Serial</span>
          {isDropdownOpen && (
            <ul className="flex flex-col w-max absolute bg-white my-1 left-0 block">
              <MenuItem text={isConnected ? "Disconnect" : "Connect"} onClick={() => handleConnect(port, baud, ending, setIsConnected)} />
              <SubMenu
                text={`Baud: ${baud}`}
                setHook={setBaud}
                menuItemList={getBaudList()}
              />
              <SubMenu
                text={`Port: ${port}`}
                setHook={setPort}
                menuItemList={portList}
              />
            </ul>
          )}
        </li>
      </ul>
    </nav>
  );
}
  return (
    <>
    
    <Navbar />
    
    <Canvas
          data={data}
          selectedBits={selectedBits}
          isGridView={isGridView}
          isDisplay={isDisplay}
        />
      <Connection
        LineData={setData}
        Connection={setIsConnected}
        selectedBits={selectedBits}
        setSelectedBits={setSelectedBits}
        isGridView={isGridView}
        setIsGridView={setIsGridView}
        isDisplay={isDisplay}
        setIsDisplay={setIsDisplay}
      />
      <Serial />
    </>
  );
};

export default Page;