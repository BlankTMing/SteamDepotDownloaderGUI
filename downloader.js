// Uses a prebuild binary of https://github.com/SteamRE/DepotDownloader
// License can be found at https://github.com/SteamRE/DepotDownloader/blob/master/LICENSE

const {
	preDownloadCheck,
	download,
	createCommand,
	runCommand,
	removeDir,
	removeFile,
	unzip
} = require("./utils")

// Initializes the variable that holds the path to the specified download location
let exportedFile

function submitForm() {
	// Check if the form is filled in and if dotnet is installed
	preDownloadCheck().then(async function () {
		document.getElementById("dotnetwarning").hidden = true
		document.getElementById("emptywarning").hidden = true
		console.info("dotnet found in PATH")

		// Remove the old depotdownloader directory if there are any
		await removeDir("depotdownloader")

		// Download a prebuild DepotDownloader binary, so it doesn't have to be included in the source code
		await download("https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_2.5.0/depotdownloader-2.5.0.zip")

		// Unzip the DepotDownloader binary
		await unzip("depotdownloader-2.5.0.zip", "depotdownloader")

		// Clean up the old files
		await removeFile("depotdownloader-2.5.0.zip")

		// Run the final command
		await runCommand(createCommand())
	}).catch(function (error) {
		if (error === "noDotnet") {
			console.error("Dotnet not found in PATH")
			document.getElementById("emptywarning").hidden = true
			document.getElementById("dotnetwarning").hidden = false
		} else if (error === "emptyField") {
			console.error("Fill in all required fields")
			document.getElementById("dotnetwarning").hidden = true
			document.getElementById("emptywarning").hidden = false
		}
	})
}

function submitDotnet() {
	const electron = require("electron")
	const os = process.platform.toString()
	document.getElementById("dotnetwarning").hidden = true
	if (os.includes("win")) {
		console.debug("Opened .NET download page for " + os.charAt(0).toUpperCase() + os.slice(1))
		void electron.shell.openExternal("https://aka.ms/dotnet/6.0/dotnet-sdk-win-x64.exe")
	}
	if (os.includes("linux")) {
		console.debug("Opened .NET download page for " + os.charAt(0).toUpperCase() + os.slice(1))
		void electron.shell.openExternal("https://docs.microsoft.com/en-us/dotnet/core/install/linux")
	}
	if (os.includes("darwin")) {
		console.debug("Opened .NET download page for" + os)
		//TODO: Apple Silicon(ARM64) URL
		void electron.shell.openExternal("https://aka.ms/dotnet/6.0/dotnet-sdk-osx-x64.pkg")
	}
}

function openGitHubIssues() {
	const electron = require("electron")
	console.debug("Opened GitHub issues page")
	void electron.shell.openExternal("https://github.com/mmvanheusden/SteamDepotDownloaderGUI/issues/new")
}

function openSteamDB() {
	const electron = require("electron")
	console.debug("Opened SteamDB instant search page")
	void electron.shell.openExternal("https://steamdb.info/instantsearch/")
}

function openDonate() {
	const electron = require("electron")
	console.debug("Opened donation page")
	void electron.shell.openExternal("https://liberapay.com/barbapapa/")
}

function checkPath() {
	// Opens the chosen location where the game will be downloaded to
	shell.openPath(exportedFile)
}

/* Everything beyond this line runs when the page is loaded */

const { ipcRenderer, shell} = require("electron")

// Add event listeners to the buttons
window.addEventListener("DOMContentLoaded", () => {
	document.getElementById("dotnetalertbtn").addEventListener("click", submitDotnet)
	document.getElementById("downloadbtn").addEventListener("click", submitForm)
	document.getElementById("smbtn1").addEventListener("click", openGitHubIssues)
	document.getElementById("smbtn2").addEventListener("click", openSteamDB)
	document.getElementById("smbtn3").addEventListener("click", openDonate)
	document.getElementById("smbtn3").addEventListener("click", openDonate)
	document.getElementById("pickpath").addEventListener("click", () => {
		ipcRenderer.send("selectpath")
	})
	document.getElementById("checkpath").addEventListener("click", checkPath)
})

ipcRenderer.on("file", (event, file) => {
	console.log("obtained file from main process: " + file)
	document.getElementById("checkpath").ariaDisabled = false // Makes the check button active
	exportedFile = file.toString()
})