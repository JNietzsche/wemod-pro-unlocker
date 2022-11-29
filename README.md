<div align="center">
  <h1>WeMod Pro Unlocker</h1>
  <h4>This program patches the WeMod app to think you're a pro subscriber.</h4>
  <img src="https://img.shields.io/crates/v/wemod-pro-unlocker?color=purple" alt="Version on Crates.io">
  <img src="https://img.shields.io/crates/d/wemod-pro-unlocker?color=red" alt="Downloads">
  <img src="https://img.shields.io/crates/l/wemod-pro-unlocker?color=green" alt="License">

  <img src="https://img.shields.io/badge/rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/github/languages/code-size/bennett-sh/wemod-pro-unlocker?color=yellow" alt="Code Size"><br/><br/>
  <img width="256" src="https://user-images.githubusercontent.com/110846042/204528128-76fc17fa-ea2c-4640-ae65-41cfba66f499.png" alt="WeMod Pro Unlocker Logo">
</div>

#### ⚠️ DISCLAIMER: Eventhough I don't think it is likely, WeMod could in theory ban you for this. I take no responsibilty for any damage caused by the usage of this program.

<br/>

## ⬇️ Installation
#### Note: after following any of the steps below, you must restart WeMod (make sure to also close it from the tray)
For installation, you have three options.
1. Download the pre-built executable [here](https://github.com/bennett-sh/wemod-pro-unlocker/releases/latest/download/wemod-pro-unlocker.exe)
2. Install Cargo, then run
```
cargo install wemod-pro-unlocker
wemod-pro-unlocker
```
3. (not recommended) Manually build it from source

<br/>

## ➕ Requirements
- [asar](https://github.com/electron/asar)

<br/>

## ❌ What does not work?
- RC From Phone (this feature is not client-side so you actually need pro; there's nothing I can do)

<br/>

## ⚙️ Configuration
| Argument                  	| Description                                                                                                                           	| Example
|---------------------------	|---------------------------------------------------------------------------------------------------------------------------------------	|----------------------------------
| --wemod-dir <dir>         	 	 	 	 	 	 	          	  | Path to your WeMod dir. By default, this is "%localappdata%/WeMod".                                         	| C:\WeMod
| --wemod-version <version> 	 	 	 	 	 	 	          	  | The version to patch. By default, this will be the latest version installed. 	                                | 8.3.6
| --asar <folder containing asar.cmd or --asar-bin>     | Path to a folder containing "asar.cmd" or the bin specified with --asar-bin.                                 	| C:\asar
| --asar-bin <file in --asar>                           | The asar executable in the folder specified in --asar (or in a default npm folder)                           	| asar.cmd
| --account <json>            	 	 	 	              	  | Overwrites the account data. You can find all available options by searching for /v3/account in the dev tools | username:'myaccount',email:'test'
| -v                          	 	 	 	 	            	  | Prints out the version info. Will cancel everything else                                                      | ---

<br/>

## 🔒 Is it safe?
This program may sound like malware at first, but if you're unsure, just read the source code. It's quite small and (hopefully, I'm relatively new to Rust) readable.

<br/>

## ❓ It stopped working after the latest update, what should I do?
This is probably because WeMod updated itself and now uses a new directory. To fix this, just run the program again.

<br/>

## 🫂 Contributors
<a href="https://github.com/bennett-sh/wemod-pro-unlocker/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=bennett-sh/wemod-pro-unlocker" />
</a>
