import React from 'react';
import '../../../styles/Modals.css';

/**
 * Settings modal for managing search paths
 */
const InfoModal = ({infoModalContent}) => (
  <div className="modal">
	<div className="modal-content">
		{infoModalContent}
	</div>
  </div>
);

export default React.memo(InfoModal);