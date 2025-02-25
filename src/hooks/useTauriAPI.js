import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Custom hook for interacting with Tauri API
 * Provides a stable reference to invoke API methods
 */
export const useTauriAPI = () => {
  const invokeAPI = useCallback(async (method, args) => {
    try {
      return await invoke(method, args);
    } catch (error) {
      console.error(`Error invoking ${method}:`, error);
      alert(error.message || "An error occurred");
      throw error;
    }
  }, []);

  return { invokeAPI };
};