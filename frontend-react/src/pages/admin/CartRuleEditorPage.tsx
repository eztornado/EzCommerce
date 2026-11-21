import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { IconCheck, IconPlus, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate } from '../../lib/format';
import { AdminPageHeader, AdminTabs } from '../../components/admin/ui';
import { t } from '../../i18n';

const ACTION_TYPES = [
  { value: 'by_percent', label: 'By percent of product price' },
  { value: 'by_fixed', label: 'By fixed amount per product' },
  { value: 'cart_fixed', label: 'Fixed amount off the whole cart' },
  { value: 'buy_x_get_y', label: 'Buy X get Y free' },
];

const OPERATORS = [
  { value: '==', label: 'is equal to' },
  { value: '!=', label: 'is not equal to' },
  { value: '>=', label: 'is greater than or equal to' },
  { value: '<=', label: 'is less than or equal to' },
  { value: '>', label: 'is greater than' },
  { value: '<', label: 'is less than' },
  { value: '{}', label: 'contains' },
  { value: '!{}', label: 'does not contain' },
];

const ATTRIBUTES = [
  { value: 'cart::base_sub_total', label: 'Cart subtotal' },
  { value: 'cart::items_qty', label: 'Cart quantity' },
  { value: 'cart::customer_id', label: 'Customer (0 = guest)' },
  { value: 'cart_item::quantity', label: 'Item quantity' },
  { value: 'cart_item::base_price', label: 'Item price' },
  { value: 'product::sku', label: 'Product SKU' },
  { value: 'product::name', label: 'Product name' },
  { value: 'product::brand', label: 'Product brand' },
];

const emptyRule: Record<string, any> = {
  name: '',
  description: '',
  status: 1,
  coupon_type: 1,
  use_auto_generation: 0,
  coupon_code: '',
  uses_per_coupon: '',
  usage_per_customer: '',
  starts_from: '',
  ends_till: '',
  condition_type: 1,
  conditions: [],
  action_type: 'by_percent',
  discount_amount: 0,
  discount_quantity: 1,
  discount_step: 1,
  apply_to_shipping: 0,
  free_shipping: 0,
  end_other_rules: 0,
  sort: 0,
  channels: [1],
  customer_groups: [1, 2, 3],
};

export default function CartRuleEditorPage() {
  const { id } = useParams();
  const isCreate = !id || id === 'create';
  const navigate = useNavigate();
  const [tab, setTab] = useState('information');
  const [rule, setRule] = useState<Record<string, any>>({ ...emptyRule });
  const [channels, setChannels] = useState<any[]>([]);
  const [groups, setGroups] = useState<any[]>([]);
  const [coupons, setCoupons] = useState<any[]>([]);
  const [loading, setLoading] = useState(!isCreate);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    api.adminGet('admin/settings/channels').then((rows) => setChannels(Array.isArray(rows) ? rows : [])).catch(() => undefined);
    api.adminGet('admin/customers/groups').then((rows) => setGroups(Array.isArray(rows) ? rows : [])).catch(() => undefined);
  }, []);

  useEffect(() => {
    if (isCreate) return;
    (async () => {
      setLoading(true);
      try {
        const data = await api.adminGet(`admin/marketing/cart-rules/view/${id}`);
        const loaded = data.rule ?? {};
        setRule({
          ...emptyRule,
          ...loaded,
          conditions: Array.isArray(loaded.conditions) ? loaded.conditions : [],
          channels: (data.channels ?? []).map((row: any) => row.channel_id),
          customer_groups: (data.customer_groups ?? []).map((row: any) => row.customer_group_id),
        });
        setCoupons(data.coupons ?? []);
      } catch (requestError: any) {
        setError(requestError?.message ?? t('Unable to load the cart rule.'));
      } finally {
        setLoading(false);
      }
    })();
  }, [id, isCreate]);

  const set = (key: string, value: any) => setRule((current) => ({ ...current, [key]: value }));

  const addCondition = () => {
    set('conditions', [...(rule.conditions ?? []), { attribute: 'cart::base_sub_total', operator: '>=', value: '' }]);
  };

  const updateCondition = (index: number, key: string, value: string) => {
    const list = [...(rule.conditions ?? [])];
    list[index] = { ...list[index], [key]: value };
    set('conditions', list);
  };

  const removeCondition = (index: number) => {
    set('conditions', (rule.conditions ?? []).filter((_: any, position: number) => position !== index));
  };

  const save = async () => {
    setError('');
    setMessage('');
    if (!String(rule.name ?? '').trim()) {
      setError(t('The name field is required.'));
      setTab('information');
      return;
    }
    setSaving(true);
    try {
      const payload = {
        ...rule,
        name: String(rule.name).trim(),
        status: Number(rule.status) ? 1 : 0,
        coupon_type: Number(rule.coupon_type),
        use_auto_generation: rule.use_auto_generation ? 1 : 0,
        apply_to_shipping: rule.apply_to_shipping ? 1 : 0,
        free_shipping: rule.free_shipping ? 1 : 0,
        end_other_rules: rule.end_other_rules ? 1 : 0,
        condition_type: Number(rule.condition_type),
        discount_amount: Number(rule.discount_amount || 0),
        discount_quantity: Number(rule.discount_quantity || 1),
        discount_step: Number(rule.discount_step || 1),
        sort: Number(rule.sort_order ?? rule.sort ?? 0),
        sort_order: Number(rule.sort_order ?? rule.sort ?? 0),
        uses_per_coupon: rule.uses_per_coupon === '' ? null : Number(rule.uses_per_coupon),
        usage_per_customer: rule.usage_per_customer === '' ? null : Number(rule.usage_per_customer),
        starts_from: rule.starts_from || null,
        ends_till: rule.ends_till || null,
        channels: rule.channels ?? [1],
        customer_groups: rule.customer_groups ?? [1, 2, 3],
        conditions: rule.conditions ?? [],
      };
      if (isCreate) {
        const created = await api.adminPost('admin/marketing/cart-rules/store', payload);
        const nextId = created?.id;
        if (nextId) navigate(`/admin/marketing/promotions/cart-rules/edit/${nextId}`);
      } else {
        await api.adminPut(`admin/marketing/cart-rules/edit/${id}`, payload);
        setMessage(t('The cart rule has been saved successfully.'));
        const data = await api.adminGet(`admin/marketing/cart-rules/view/${id}`);
        setCoupons(data.coupons ?? []);
      }
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the cart rule.'));
    } finally {
      setSaving(false);
    }
  };

  const generateCoupons = async () => {
    if (isCreate) return;
    try {
      await api.adminPost(`admin/marketing/cart-rules/coupons/${id}`, { count: 10 });
      const data = await api.adminGet(`admin/marketing/cart-rules/view/${id}`);
      setCoupons(data.coupons ?? []);
      setMessage('10 coupon codes have been generated.');
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to generate coupons.'));
    }
  };

  if (loading) {
    return (
      <div>
        <AdminPageHeader title={t('Loading cart rule…')} backTo="/admin/marketing/promotions/cart-rules" />
        <div className="admin-card admin-card-pad">
          {Array.from({ length: 5 }).map((_, index) => (
            <div className="admin-skeleton-row" key={index} />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <AdminPageHeader
        title={isCreate ? t('Create cart rule') : t('Edit cart rule · {name}', { name: rule.name })}
        subtitle={t('Configure conditions, coupon behaviour and the discount action.')}
        backTo="/admin/marketing/promotions/cart-rules"
        actions={
          <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
            <IconCheck size={15} /> {saving ? t('Saving…') : t('Save cart rule')}
          </button>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card">
        <AdminTabs
          tabs={[
            { key: 'information', label: 'Information' },
            { key: 'conditions', label: 'Conditions' },
            { key: 'actions', label: 'Actions' },
            { key: 'coupons', label: 'Coupons', count: coupons.length },
          ]}
          active={tab}
          onChange={setTab}
        />
        <div className="admin-card-pad">
          {tab === 'information' && (
            <div className="admin-form-grid">
              <label className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">
                  {t('Name')} <em>*</em>
                </span>
                <input className="admin-input" value={rule.name ?? ''} onChange={(event) => set('name', event.target.value)} placeholder={t('Summer sale 10%')} />
              </label>
              <label className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">{t('Description')}</span>
                <textarea className="admin-input" rows={3} value={rule.description ?? ''} onChange={(event) => set('description', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Status')}</span>
                <select className="admin-input" value={rule.status ? '1' : '0'} onChange={(event) => set('status', event.target.value)}>
                  <option value="1">{t('Active')}</option>
                  <option value="0">{t('Inactive')}</option>
                </select>
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Coupon type')}</span>
                <select className="admin-input" value={rule.coupon_type ?? 1} onChange={(event) => set('coupon_type', event.target.value)}>
                  <option value="1">{t('No coupon')}</option>
                  <option value="2">{t('Specific coupon')}</option>
                </select>
              </label>
              {Number(rule.coupon_type) === 2 && (
                <>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('Coupon code')}</span>
                    <input className="admin-input" value={rule.coupon_code ?? ''} onChange={(event) => set('coupon_code', event.target.value)} placeholder={t('WELCOME10')} />
                  </label>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('Auto-generate coupon codes')}</span>
                    <span className="admin-switch-row">
                      <input type="checkbox" checked={!!rule.use_auto_generation} onChange={(event) => set('use_auto_generation', event.target.checked)} />
                      <span>{rule.use_auto_generation ? t('Enabled') : t('Disabled')}</span>
                    </span>
                  </label>
                </>
              )}
              <label className="admin-field">
                <span className="admin-field-label">{t('Start date')}</span>
                <input className="admin-input" type="date" value={(rule.starts_from ?? '').slice(0, 10)} onChange={(event) => set('starts_from', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('End date')}</span>
                <input className="admin-input" type="date" value={(rule.ends_till ?? '').slice(0, 10)} onChange={(event) => set('ends_till', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Uses per coupon')}</span>
                <input className="admin-input" type="number" min={0} value={rule.uses_per_coupon ?? ''} onChange={(event) => set('uses_per_coupon', event.target.value)} placeholder={t('Unlimited')} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Uses per customer')}</span>
                <input className="admin-input" type="number" min={0} value={rule.usage_per_customer ?? ''} onChange={(event) => set('usage_per_customer', event.target.value)} placeholder={t('Unlimited')} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Priority')}</span>
                <input className="admin-input" type="number" value={rule.sort ?? 0} onChange={(event) => set('sort', event.target.value)} />
              </label>
              <div className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">{t('Channels')}</span>
                <div className="admin-check-grid">
                  {channels.map((channel) => (
                    <label key={channel.id} className="admin-check-row">
                      <input
                        type="checkbox"
                        checked={(rule.channels ?? []).includes(channel.id)}
                        onChange={() =>
                          set(
                            'channels',
                            (rule.channels ?? []).includes(channel.id)
                              ? (rule.channels ?? []).filter((value: number) => value !== channel.id)
                              : [...(rule.channels ?? []), channel.id],
                          )
                        }
                      />
                      <span>{channel.name ?? channel.code}</span>
                    </label>
                  ))}
                </div>
              </div>
              <div className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">{t('Customer groups')}</span>
                <div className="admin-check-grid">
                  {groups.map((group) => (
                    <label key={group.id} className="admin-check-row">
                      <input
                        type="checkbox"
                        checked={(rule.customer_groups ?? []).includes(group.id)}
                        onChange={() =>
                          set(
                            'customer_groups',
                            (rule.customer_groups ?? []).includes(group.id)
                              ? (rule.customer_groups ?? []).filter((value: number) => value !== group.id)
                              : [...(rule.customer_groups ?? []), group.id],
                          )
                        }
                      />
                      <span>{group.name}</span>
                    </label>
                  ))}
                </div>
              </div>
            </div>
          )}

          {tab === 'conditions' && (
            <div>
              <div className="admin-form-grid" style={{ marginBottom: 18 }}>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Condition combination')}</span>
                  <select className="admin-input" value={rule.condition_type ?? 1} onChange={(event) => set('condition_type', event.target.value)}>
                    <option value="1">{t('All conditions must be true (AND)')}</option>
                    <option value="2">{t('Any condition must be true (OR)')}</option>
                  </select>
                </label>
              </div>
              <div className="admin-condition-list">
                {(rule.conditions ?? []).length === 0 && (
                  <p className="admin-muted">{t('No conditions. The rule will apply to every cart.')}</p>
                )}
                {(rule.conditions ?? []).map((condition: any, index: number) => (
                  <div className="admin-condition-row" key={index}>
                    <select
                      className="admin-input"
                      value={condition.attribute ?? ''}
                      onChange={(event) => updateCondition(index, 'attribute', event.target.value)}
                    >
                      {ATTRIBUTES.map((attribute) => (
                        <option key={attribute.value} value={attribute.value}>
                          {t(attribute.label)}
                        </option>
                      ))}
                    </select>
                    <select
                      className="admin-input"
                      value={condition.operator ?? '=='}
                      onChange={(event) => updateCondition(index, 'operator', event.target.value)}
                    >
                      {OPERATORS.map((operator) => (
                        <option key={operator.value} value={operator.value}>
                          {t(operator.label)}
                        </option>
                      ))}
                    </select>
                    <input
                      className="admin-input"
                      value={condition.value ?? ''}
                      onChange={(event) => updateCondition(index, 'value', event.target.value)}
                      placeholder={t('Value')}
                    />
                    <button type="button" className="admin-icon-action danger" onClick={() => removeCondition(index)}>
                      <IconTrash size={15} />
                    </button>
                  </div>
                ))}
              </div>
              <button type="button" className="admin-button" onClick={addCondition} style={{ marginTop: 12 }}>
                <IconPlus size={15} /> {t('Add condition')}
              </button>
            </div>
          )}

          {tab === 'actions' && (
            <div className="admin-form-grid">
              <label className="admin-field">
                <span className="admin-field-label">{t('Action type')}</span>
                <select className="admin-input" value={rule.action_type ?? 'by_percent'} onChange={(event) => set('action_type', event.target.value)}>
                  {ACTION_TYPES.map((action) => (
                    <option key={action.value} value={action.value}>
                      {t(action.label)}
                    </option>
                  ))}
                </select>
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Discount amount')}</span>
                <input className="admin-input" type="number" step="0.01" value={rule.discount_amount ?? 0} onChange={(event) => set('discount_amount', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Discount quantity')}</span>
                <input className="admin-input" type="number" value={rule.discount_quantity ?? 1} onChange={(event) => set('discount_quantity', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Discount step (buy X)')}</span>
                <input className="admin-input" type="number" value={rule.discount_step ?? 1} onChange={(event) => set('discount_step', event.target.value)} />
              </label>
              <label className="admin-check-row">
                <input type="checkbox" checked={!!rule.apply_to_shipping} onChange={(event) => set('apply_to_shipping', event.target.checked)} />
                <span>{t('Apply to shipping amount')}</span>
              </label>
              <label className="admin-check-row">
                <input type="checkbox" checked={!!rule.free_shipping} onChange={(event) => set('free_shipping', event.target.checked)} />
                <span>{t('Free shipping')}</span>
              </label>
              <label className="admin-check-row">
                <input type="checkbox" checked={!!rule.end_other_rules} onChange={(event) => set('end_other_rules', event.target.checked)} />
                <span>{t('Stop further rules from processing')}</span>
              </label>
            </div>
          )}

          {tab === 'coupons' && (
            <div>
              {isCreate ? (
                <div className="admin-empty">{t('Save the rule before generating coupons.')}</div>
              ) : (
                <>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14 }}>
                    <p className="admin-muted">{t('Coupon codes attached to this rule.')}</p>
                    <button type="button" className="admin-button" onClick={generateCoupons}>
                      <IconPlus size={15} /> {t('Generate 10 coupons')}
                    </button>
                  </div>
                  {coupons.length === 0 ? (
                    <div className="admin-empty">{t('No coupons have been generated for this rule.')}</div>
                  ) : (
                    <table className="admin-table">
                      <thead>
                        <tr>
                          <th>{t('Code')}</th>
                          <th>{t('Usage limit')}</th>
                          <th>{t('Per customer')}</th>
                          <th>{t('Times used')}</th>
                          <th>{t('Expires')}</th>
                          <th>{t('Primary')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {coupons.map((coupon) => (
                          <tr key={coupon.id}>
                            <td>
                              <span className="admin-code">{coupon.code}</span>
                            </td>
                            <td>{coupon.usage_limit ?? t('Unlimited')}</td>
                            <td>{coupon.usage_per_customer ?? t('Unlimited')}</td>
                            <td>{coupon.times_used ?? 0}</td>
                            <td>{coupon.expired_at ? formatDate(coupon.expired_at) : '—'}</td>
                            <td>{coupon.is_primary ? <span className="status-badge green">{t('Primary')}</span> : '—'}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
