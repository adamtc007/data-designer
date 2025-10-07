// Stub for Tauri API when running in browser mode
export const invoke = async () => {
    throw new Error('Tauri API not available in browser mode');
};

export default {
    invoke
};