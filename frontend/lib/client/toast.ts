/**
 * Simple toast notification utility
 * Client-side only
 */

'use client';

export type ToastType = 'success' | 'error' | 'info';

interface ToastOptions {
  message: string;
  type?: ToastType;
  duration?: number;
}

let toastContainer: HTMLDivElement | null = null;

function getToastContainer(): HTMLDivElement {
  if (!toastContainer) {
    toastContainer = document.createElement('div');
    toastContainer.id = 'toast-container';
    toastContainer.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 9999;
      display: flex;
      flex-direction: column;
      gap: 8px;
      pointer-events: none;
    `;
    document.body.appendChild(toastContainer);
  }
  return toastContainer;
}

function getToastStyles(type: ToastType): string {
  const baseStyles = `
    padding: 12px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    pointer-events: auto;
    animation: slideIn 0.3s ease-out;
  `;

  const typeStyles = {
    success: 'background-color: #10b981; color: white;',
    error: 'background-color: #ef4444; color: white;',
    info: 'background-color: #3b82f6; color: white;',
  };

  return baseStyles + typeStyles[type];
}

export function showToast(options: ToastOptions): void {
  const { message, type = 'info', duration = 3000 } = options;

  const container = getToastContainer();
  const toast = document.createElement('div');
  toast.style.cssText = getToastStyles(type);
  toast.textContent = message;

  container.appendChild(toast);

  // Remove after duration
  setTimeout(() => {
    toast.style.animation = 'slideOut 0.3s ease-out';
    setTimeout(() => {
      container.removeChild(toast);
      
      // Clean up container if empty
      if (container.children.length === 0 && toastContainer) {
        document.body.removeChild(toastContainer);
        toastContainer = null;
      }
    }, 300);
  }, duration);
}

// Add animations to document if not already present
if (typeof document !== 'undefined') {
  const styleId = 'toast-animations';
  if (!document.getElementById(styleId)) {
    const style = document.createElement('style');
    style.id = styleId;
    style.textContent = `
      @keyframes slideIn {
        from {
          transform: translateX(100%);
          opacity: 0;
        }
        to {
          transform: translateX(0);
          opacity: 1;
        }
      }
      
      @keyframes slideOut {
        from {
          transform: translateX(0);
          opacity: 1;
        }
        to {
          transform: translateX(100%);
          opacity: 0;
        }
      }
    `;
    document.head.appendChild(style);
  }
}
