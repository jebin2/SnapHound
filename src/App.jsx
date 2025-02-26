import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

// Import custom hook
import { useTauriAPI } from "./hooks/useTauriAPI";

// Import components
import Header from "./components/Header";
import MediaGrid from "./components/MediaGrid";
import StatusBar from "./components/StatusBar";
import { SettingsModal, VideoModal, InfoModal } from "./components/Modals";

// Import constants and utilities
import { SEARCH_TYPES } from "./constants";
import { deduplicateMediaItems } from "./utils/mediaUtils";

// Import styles
import "./styles/App.css";

function App() {
	// State management
	const [status, setStatus] = useState("Initializing...");
	const [indexStatus, setIndexStatus] = useState("Indexing...");
	const [searchType, setSearchType] = useState(SEARCH_TYPES.IMAGE);
	const [searchValue, setSearchValue] = useState("");
	const [settingsOpen, setSettingsOpen] = useState(false);
	const [videoModalOpen, setVideoModalOpen] = useState(false);
	const [infoModalOpen, setInfoModalOpen] = useState(true);
	const [infoModalContent, setInfoModalContent] = useState("Initialising setup...");
	const [mediaItems, setMediaItems] = useState([]);
	const [searchPaths, setSearchPaths] = useState([]);

	// Use custom Tauri API hook
	const { invokeAPI } = useTauriAPI();

	// Environment initialization
	useEffect(() => {
		invokeAPI("initialize_environment");
	}, [invokeAPI]);

	// Event listeners setup
	useEffect(() => {
		const unlisten = [];

		const setupListeners = async () => {
			try {
				// Listen for status updates from Rust
				unlisten.push(await listen("status_update", ({ payload }) => {
					setStatus(payload);
				}));
				unlisten.push(await listen("index_status", ({ payload }) => {
					setIndexStatus(payload);
				}));

				// Listen for new file paths
				unlisten.push(await listen("file_path", ({ payload }) => {
					if (!searchValueRef.current) {
						const newItems = JSON.parse(payload);
						setMediaItems(prev => deduplicateMediaItems(prev, newItems));
					}
				}));

				// Listen for end of file paths
				unlisten.push(await listen("file_path_end", () => {
					setMediaItems(prev => prev.length ? prev : [{ id: "empty" }]);
				}));

				// Listen for fetch list trigger
				unlisten.push(await listen("can_fetch_list", () => {
					if (!searchValueRef.current) {
						handleCloseInfoModal();
						invokeAPI("list_files");
					}
				}));

				unlisten.push(await listen("success_reset", () => {
					setInfoModalContent("App will restart.");
					setInfoModalOpen(true);
					setTimeout(() => {
						// invokeAPI("relaunch");
						location.reload();
					}, 1000);
				}));

				unlisten.push(await listen("searched_result", ({ payload }) => {
					if (searchValueRef.current) {
						const newItems = JSON.parse(payload);
						setMediaItems(prev => deduplicateMediaItems(prev, newItems));
					}
				}));
				unlisten.push(await listen("remove_all_data", () => {
					setMediaItems([]);
				}));
			} catch (error) {
				console.error("Error setting up listeners:", error);
			}
		};

		setupListeners();

		// Cleanup listeners on unmount
		return () => unlisten.forEach(fn => fn?.());
	}, [invokeAPI]);

	// Settings management
	const fetchConfig = useCallback(async () => {
		try {
			const config = await invokeAPI("fetch_config");
			setSearchPaths(config?.priority_paths || []);
			setSettingsOpen(true);
		} catch (error) {
			console.error("Failed to fetch config:", error);
		}
	}, [invokeAPI]);

	const saveConfig = useCallback(async () => {
		try {
			await invokeAPI("save_config", { priorityPath: searchPaths });
			setSettingsOpen(false);
		} catch (error) {
			console.error("Failed to save config:", error);
		}
	}, [invokeAPI, searchPaths]);

	// Path management
	const handleAddPath = useCallback(async () => {
		try {
			const folderPath = await invokeAPI("select_folder");
			if (folderPath) {
				setSearchPaths(prev => [...prev, folderPath]);
			}
		} catch (error) {
			console.error("Failed to select folder:", error);
		}
	}, [invokeAPI]);

	const handleRemovePath = useCallback((index) =>
		setSearchPaths(prev => prev.filter((_, i) => i !== index)),
		[]);

	// Event handlers
	const handleSearchTypeChange = useCallback((type) => {
		setSearchType(type);
	}, []);

	const onSearchInputChange = useCallback(async (event) => {
		let value = event.target.value;
		setSearchValue(value);
	
		if (!value || value.length <= 2) {
			// Clear any existing timeout before setting a new one
			if (window.searchTimeout) {
				clearTimeout(window.searchTimeout);
			}
	
			// Wait for user to stop typing, then call list_files
			window.searchTimeout = setTimeout(async () => {
				invokeAPI("list_files");
			}, 500); // Adjust delay as needed
	
			return;
		}
	
		// Clear previous timeout if user types again
		if (window.searchTimeout) {
			clearTimeout(window.searchTimeout);
		}
	
		// Set a new timeout to wait before executing search
		window.searchTimeout = setTimeout(async () => {
			invokeAPI("search_indexed_data", { searchQuery: value });
		}, 500); // Adjust delay as needed
	}, [setSearchValue, invokeAPI]);
	const searchValueRef = useRef(searchValue);

	useEffect(() => {
		searchValueRef.current = searchValue;
	}, [searchValue]);
	

	const handleResetAll = useCallback(async () => {
		// Implement reset logic
		await invokeAPI("reset_all");
		handleCloseSettings()
	}, []);

	const handleOpenSettings = useCallback(() => fetchConfig(), [fetchConfig]);
	const handleCloseSettings = useCallback(() => setSettingsOpen(false), []);
	const handleOpenVideoModal = useCallback(() => setVideoModalOpen(true), []);
	const handleCloseVideoModal = useCallback(() => setVideoModalOpen(false), []);
	const handleCloseInfoModal = useCallback(() => setInfoModalOpen(false), []);

	return (
		<>
			<div className="container">
				<Header
					onSearchInputChange={onSearchInputChange}
					searchType={searchType}
					searchValue={searchValue}
					onSearchTypeChange={handleSearchTypeChange}
					openSettings={handleOpenSettings}
					openVideoModal={handleOpenVideoModal}
				/>

				<div className="progress-bar">
					<div className="progress-fill" />
				</div>

				<MediaGrid mediaItems={mediaItems} />

				{videoModalOpen && <VideoModal onClose={handleCloseVideoModal} />}
				{infoModalOpen && <InfoModal infoModalContent={infoModalContent}/>}

				{settingsOpen && (
					<SettingsModal
						searchPaths={searchPaths}
						onClose={handleCloseSettings}
						onAddPath={handleAddPath}
						onRemovePath={handleRemovePath}
						onSave={saveConfig}
						handleResetAll={handleResetAll}
					/>
				)}
			</div>

			{/* Status Bar */}
			<StatusBar status={status} indexStatus={indexStatus} />
		</>
	);
}

export default App;