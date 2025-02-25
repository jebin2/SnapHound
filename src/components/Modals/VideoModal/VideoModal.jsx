import React from 'react';
import '../../../styles/Modals.css';

/**
 * Video modal for video search functionality
 */
const VideoModal = ({ onClose }) => (
  <div className="modal">
    <div className="modal-content">
      <h2>Video Search</h2>
      <p>This may take some time. Select videos to analyze:</p>
      <button className="start-analysis">Start Analysis</button>
      <button className="close-modal" onClick={onClose}>
        Close
      </button>
    </div>
  </div>
);

export default React.memo(VideoModal);