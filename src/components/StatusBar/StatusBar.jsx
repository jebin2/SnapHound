import React from 'react';
import '../../styles/StatusBar.css';

/**
 * StatusBar component to display current application status
 */
const StatusBar = ({ status }) => (
  <div className="status-bar">
    {status || "Idle..."}
  </div>
);

export default React.memo(StatusBar);