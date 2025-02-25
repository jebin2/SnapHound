import React from 'react';
import '../../../styles/Modals.css';

/**
 * Settings modal for managing search paths
 */
const SettingsModal = ({
	searchPaths,
	onClose,
	onAddPath,
	onRemovePath,
	onSave,
	handleResetAll
}) => (
	<div className="modal">
		<div className="modal-content">
			<h2>Manage Search Paths</h2>
			<div className="path-list">
				{searchPaths.map((path, index) => (
					<div key={`${path}-${index}`} className="path-item">
						<input className="path-input" value={path} readOnly />
						<button
							className="remove-path"
							onClick={() => onRemovePath(index)}
						>
							✕
						</button>
					</div>
				))}
			</div>

			<button className="add-path-btn" onClick={onAddPath}>
				＋ Add New Path
			</button>

			<div className="modal-actions">
				<button className="search-type-btn" onClick={handleResetAll}>
					Reset All
				</button>
				<button className="search-type-btn" onClick={onClose}>
					Cancel
				</button>
				<button className="search-type-btn" onClick={onSave}>
					Save Changes
				</button>
			</div>
		</div>
	</div>
);

export default React.memo(SettingsModal);