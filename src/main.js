const { invoke, convertFileSrc } = window.__TAURI__.core;
const listen = window.__TAURI__.event.listen;

async function invokeAPI(method, ...args) {
	try {
		return await invoke(method, ...args);
	} catch (error) {
		console.error(`Error invoking ${method}:`, error);
		alert(`${error}`);
		throw error;
	}
}

/** Event Listeners **/
// listen('file_path', (event) => populateMedia(JSON.parse(event.payload)));
listen('file_path_end', (event) => {
	const grid = document.querySelector('.media-grid');
    if (grid.innerHTML.trim() === "Loading...") {
        grid.innerHTML = "Add paths from settings.";
	}
});
listen('can_fetch_list', () => invokeAPI("list_files"));

/** UI Elements **/
const settingsModal = document.getElementById('SettingsModal');
const videoModal = document.getElementById('videoModal');
const mediaGrid = document.querySelector('.media-grid');
const mediaContainer = document.querySelector('.media-container');
const searchTypeButtons = document.querySelectorAll('.search-type-btn');
const settingIcon = document.getElementById('setting_icon');
const addPathBtn = document.getElementById('addPathBtn');
const settingSaveBtn = document.getElementById('settingSave');
const settingCancelBtn = document.getElementById('settingCancel');

/**
 * Handles media type switching (Images/Videos)
 */
searchTypeButtons.forEach(btn => {
	btn.addEventListener('click', () => {
        searchTypeButtons.forEach(b => b.classList.remove('active'));
		btn.classList.add('active');
	});
});

/**
 * Opens the video search modal
 */
document.querySelector('[data-type="video"]').addEventListener('click', () => {
    videoModal.style.display = 'grid';
});

/**
 * Opens the settings modal and loads search paths
 */
settingIcon.addEventListener('click', async () => {
    settingsModal.style.display = 'grid';
    const config = await invokeAPI("fetch_config");
	populatePaths(config.priority_paths);
});

/**
 * Populates the search path list inside settings modal
 */
function populatePaths(paths) {
    const container = document.getElementById('pathList');
    container.innerHTML = '';
    paths.forEach((path, index) => {
        const div = document.createElement('div');
        div.className = 'path-item';
        div.innerHTML = `
            <input class="path-input" value="${path}" data-index="${index}">
            <button class="remove-path" onclick="removePath(${index})">✕</button>
        `;
        container.appendChild(div);
    });
}

/**
 * Removes a search path from the list
 */
function removePath(index) {
    const inputs = [...document.querySelectorAll('.path-input')];
    inputs.splice(index, 1);
    populatePaths(inputs.map(input => input.value));
}

/**
 * Adds a new search path
 */
addPathBtn.addEventListener('click', async () => {
	const folderPath = await invokeAPI("select_folder");
    if (folderPath) {
        const currentPaths = [...document.querySelectorAll('.path-input')].map(input => input.value);
        populatePaths([...currentPaths, folderPath]);
    }
});

/**
 * Saves search paths to config
 */
settingSaveBtn.addEventListener('click', async () => {
    const paths = [...document.querySelectorAll('.path-input')]
        .map(input => input.value.trim())
        .filter(path => path.length > 0);
    
		await invokeAPI("save_config", { priorityPath: paths });
    settingsModal.style.display = 'none';
});

/**
 * Closes settings modal on cancel
 */
settingCancelBtn.addEventListener('click', () => {
    settingsModal.style.display = 'none';
});

/**
 * Updates progress bar
 */
function updateProgress(percent) {
	document.querySelector('.progress-fill').style.width = `${percent}%`;
}

/**
 * Populates media grid dynamically
 */
// function populateMedia(item_arr) {
//     if (document.querySelectorAll(".media-item").length < 50) {
//         item_arr.forEach(item => {
//             const mediaItem = document.createElement('div');
//             mediaItem.classList.add('media-item');
//             const assetUrl = convertFileSrc(item.path);
            
//             const img = document.createElement('img');
//             img.id = item.id;
//             img.src = assetUrl;
//             img.alt = item.name;
//             img.loading = "lazy";
//             img.onerror = () => {
//                 console.error('Failed to load image:', item.path);
//                 img.style.display = 'none';
//             };
        
//             if (item.type === 'video') {
//                 const videoIndicator = document.createElement('div');
//                 videoIndicator.classList.add('video-indicator');
//                 videoIndicator.textContent = '▶';
//                 mediaItem.appendChild(videoIndicator);
//             }
//             mediaItem.appendChild(img);
//             mediaGrid.appendChild(mediaItem);
//         });
//     }
// }

/**
 * Initializes environment on page load
 */
window.addEventListener('DOMContentLoaded', () => invokeAPI("initialize_environment"));

/**
 * Closes video modal on clicking outside
 */
document.addEventListener('click', (event) => {
    if (!event.target.dataset || event.target.dataset.type !== "video") {
        videoModal.style.display = "none";
	}
});



let allMediaItems = []; // Store all items received from backend

/** Event Listeners **/
listen('file_path', (event) => {
    const newItems = JSON.parse(event.payload); // 10 items at a time
    allMediaItems = allMediaItems.concat(newItems); // Accumulate items
    populateMedia(newItems);
});

function populateMedia(item_arr) {
    const existingItems = document.querySelectorAll('.media-item').length;

    // if (existingItems < 500) {
        item_arr.forEach(item => {
            const mediaItem = createMediaElement(item);
            mediaGrid.appendChild(mediaItem);
        });
    // }
}

function createMediaElement(item) {
    const mediaItem = document.createElement('div');
    mediaItem.classList.add('media-item');
    const assetUrl = convertFileSrc(item.path);

    const img = document.createElement('img');
    img.id = item.id;
    img.src = assetUrl;
    img.alt = item.name;
    img.loading = 'lazy';
    img.style.width = '200px';
    img.style.height = '200px';
    img.onerror = () => {
        console.error('Failed to load image:', item.path);
        img.style.display = 'none';
    };

    if (item.type === 'video') {
        const videoIndicator = document.createElement('div');
        videoIndicator.classList.add('video-indicator');
        videoIndicator.textContent = '▶';
        mediaItem.appendChild(videoIndicator);
    }
    mediaItem.appendChild(img);
    return mediaItem;
}