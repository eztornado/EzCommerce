import { useEffect } from 'react';
import { createPortal } from 'react-dom';
import { IconX } from '@tabler/icons-react';

interface Props {
  open: boolean;
  onClose: () => void;
  title?: string;
  size?: 'md' | 'lg';
  children: React.ReactNode;
}

/** Lightweight storefront modal (avoids loading Mantine on the shop). */
export default function ShopModal({ open, onClose, title, size = 'md', children }: Props) {
  useEffect(() => {
    if (!open) return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => {
      document.body.style.overflow = previous;
      window.removeEventListener('keydown', onKey);
    };
  }, [open, onClose]);

  if (!open) return null;

  return createPortal(
    <div
      className="shop-modal-overlay"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        className={`shop-modal${size === 'lg' ? ' shop-modal-lg' : ''}`}
        role="dialog"
        aria-modal="true"
        aria-label={title}
      >
        <div className="shop-modal-head">
          <h3>{title}</h3>
          <button type="button" className="shop-modal-close" onClick={onClose} aria-label="Close">
            <IconX size={18} />
          </button>
        </div>
        <div className="shop-modal-body">{children}</div>
      </div>
    </div>,
    document.body,
  );
}
