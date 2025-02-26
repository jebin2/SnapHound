import React from 'react';
import '../../styles/StatusBar.css';

/**
 * StatusBar component to display current application status
 */
const StatusBar = ({ status, indexStatus }) => (
  <div className="status-container">
    <div className="status-bar">{status || "Idle..."}</div>
    <div className="index-status-bar">{indexStatus || "Idle..."}</div>
  </div>
);

export default React.memo(StatusBar);
