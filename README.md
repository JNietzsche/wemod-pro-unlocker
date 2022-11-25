<div align="center">
  <h1>WeMod Pro Unlocker</h1>
  <h4>This program patches the WeMod app to think you're a pro subscriber.</h4>
  <img src="https://img.shields.io/crates/v/wemod-pro-unlocker?color=purple" alt="Version on Crates.io">
  <img src="https://img.shields.io/crates/d/wemod-pro-unlocker?color=red" alt="Downloads">
  <img src="https://img.shields.io/crates/l/wemod-pro-unlocker?color=green" alt="License">
  <img src="https://img.shields.io/badge/rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/github/languages/code-size/bennett-sh/wemod-pro-unlocker?color=yellow" alt="Code Size"><br/><br/>
  <img width="256" src="https://user-images.githubusercontent.com/110846042/199363901-4b212629-895c-47a5-a059-4df198b64565.png" alt="WeMod Logo">
</div>

### ⚠️ DISCLAIMER: Eventhough I don't think it is likely, WeMod could in theory ban you for this. I take no responsibilty for any damage caused by the usage of this program.

<br/>

## ⬇️ Installation
#### Note: after following any of the steps below, you must restart WeMod (make sure to also close it from the tray)
For installation, you have two options.
1. Install Cargo, then run
```
cargo install wemod-pro-unlocker
wemod-pro-unlocker
```
2. (not recommended) Manually build it from source

<br/>

## ❌ What does not work?
- Saving Mods (this was available until WeMod v8.3.9 broke it, so you can download v8.3.8 from [here](https://storage-cdn.wemod.com/app/releases/stable/WeMod-8.3.8.exe))
- RC From Phone (this feature is not client-side so you actually need pro; there's nothing I can do)

<br/>

## ⚙️ Configuration
| Argument                  	| Description                                                                                                 	| Example
|---------------------------	|-------------------------------------------------------------------------------------------------------------	|----------------------------------
| --wemod-dir <dir>         	 	 	 	 	 	 	| Path to your WeMod dir. By default, this is "%localappdata%/WeMod".                                         	| C:\WeMod
| --wemod-version <version> 	 	 	 	 	 	 	| The version to patch. By default, this will be the latest version installed. 	                                | 8.3.6
| --asar-bin <folder containing asar.cmd> | Path to a folder containing "asar.cmd".                                                                     	| C:\asar
| --account <json>            	 	 	 	 	  | Overwrites the account data. You can find all available options by searching for /v3/account in the dev tools | username:'myaccount',email:'test'

<br/>

## 🔒 Is it safe?
This program may sound like malware at first, but if you're unsure, just read the source code. It's quite small and (hopefully, I'm relatively new to Rust) readable.

<br/>

## ❓ It stopped working after the latest update, what should I do?
This is probably because WeMod updated itself and now uses a new directory. To fix this, just run the program again.
