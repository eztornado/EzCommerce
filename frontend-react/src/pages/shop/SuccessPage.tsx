import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { IconCircleCheck, IconTruck, IconMail, IconArrowRight } from '@tabler/icons-react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { formatDate, formatLabel } from '../../lib/format';
import { t } from '../../i18n';

export default function SuccessPage() {
  const { id } = useParams();
  const [order, setOrder] = useState<any>(null);

  useEffect(() => {
    if (!id) return;
    api
      .get(`checkout/onepage/success/${id}${shopQuery()}`)
      .then((data) => setOrder(data))
      .catch(() => setOrder(null));
  }, [id]);

  return (
    <div className="shop-container">
      <div className="success-shell">
        <div className="success-icon">
          <IconCircleCheck size={44} />
        </div>
        <span className="listing-eyebrow">{t('Order confirmed')}</span>
        <h1 className="shop-heading" style={{ fontSize: 40 }}>
          {t('Thank you for your order!')}
        </h1>
        <p className="shop-muted" style={{ maxWidth: 560, margin: '12px auto 0', lineHeight: 1.7 }}>
          {t('Your order has been placed successfully. We have sent a confirmation email with all the details. You can follow the progress of your order from your account.')}
        </p>

        {order && (
          <div className="success-card">
            <div className="success-row">
              <span>{t('Order number')}</span>
              <strong>#{order.increment_id}</strong>
            </div>
            <div className="success-row">
              <span>{t('Date')}</span>
              <strong>{formatDate(order.created_at, true)}</strong>
            </div>
            <div className="success-row">
              <span>{t('Status')}</span>
              <span className="status-badge yellow" style={{ textTransform: 'capitalize' }}>
                {order.status}
              </span>
            </div>
            <div className="success-row">
              <span>{t('Payment')}</span>
              <strong style={{ textTransform: 'capitalize' }}>{order.payment_method_title ?? order.payment_method ?? t('Cash on delivery')}</strong>
            </div>
            <div className="success-row">
              <span>{t('Shipping')}</span>
              <strong>{order.shipping_title ?? order.shipping_description ?? (formatLabel(order.shipping_method) || t('Standard shipping'))}</strong>
            </div>
            <div className="success-row">
              <span>{t('Total')}</span>
              <strong>€{Number(order.grand_total ?? 0).toFixed(2)}</strong>
            </div>
          </div>
        )}

        <div className="success-actions">
          <Link to="/account/orders" className="shop-primary-button">
            {t('View my orders')} <IconArrowRight size={16} />
          </Link>
          <Link to="/search" className="shop-secondary-button">
            {t('Continue shopping')}
          </Link>
        </div>

        <div className="success-assurances">
          <div>
            <IconMail size={20} />
            <span>{t('A confirmation email is on its way.')}</span>
          </div>
          <div>
            <IconTruck size={20} />
            <span>{t('Orders are dispatched within 1–2 business days.')}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
