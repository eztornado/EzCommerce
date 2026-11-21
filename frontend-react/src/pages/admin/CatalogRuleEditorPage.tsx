import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { IconCheck, IconPlus, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader, AdminTabs } from '../../components/admin/ui';
import { t } from '../../i18n';

const ACTION_TYPES = [
  { value: 'by_percent', label: 'By percent of product price' },
  { value: 'by_fixed', label: 'By fixed amount' },
  { value: 'to_percent', label: 'To percent of product price' },
  { value: 'to_fixed', label: 'To fixed amount' },
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
  { value: 'product::sku', label: 'Product SKU' },
  { value: 'product::name', label: 'Product name' },
  { value: 'product::price', label: 'Product price' },
  { value: 'product::brand', label: 'Product brand' },
  { value: 'product::category_ids', label: 'Category' },
];

const emptyRule: Record<string, any> = {
  name: '',
  description: '',
  status: 1,
  starts_from: '',
  ends_till: '',
  condition_type: 1,
  conditions: [],
  action_type: 'by_percent',
  discount_amount: 0,
  sort_order: 0,
  channels: [1],
  customer_groups: [1, 2, 3],
};

export default function CatalogRuleEditorPage() {
  const { id } = useParams();
  const isCreate = !id || id === 'create';
  const navigate = useNavigate();
  const [tab, setTab] = useState('information');
  const [rule, setRule] = useState<Record<string, any>>({ ...emptyRule });
  const [channels, setChannels] = useState<any[]>([]);
  const [groups, setGroups] = useState<any[]>([]);
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
        const data = await api.adminGet(`admin/marketing/catalog-rules/view/${id}`);
        setRule({
          ...emptyRule,
          ...(data ?? {}),
          conditions: Array.isArray(data?.conditions) ? data.conditions : [],
        });
      } catch (requestError: any) {
        setError(requestError?.message ?? t('Unable to load the catalog rule.'));
      } finally {
        setLoading(false);
      }
    })();
  }, [id, isCreate]);

  const set = (key: string, value: any) => setRule((current) => ({ ...current, [key]: value }));

  const updateCondition = (index: number, key: string, value: string) => {
    const list = [...(rule.conditions ?? [])];
    list[index] = { ...list[index], [key]: value };
    set('conditions', list);
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
        condition_type: Number(rule.condition_type),
        discount_amount: Number(rule.discount_amount || 0),
        sort_order: Number(rule.sort_order || 0),
        sort: Number(rule.sort_order || 0),
        starts_from: rule.starts_from || null,
        ends_till: rule.ends_till || null,
        conditions: rule.conditions ?? [],
        channels: rule.channels ?? [1],
        customer_groups: rule.customer_groups ?? [1, 2, 3],
      };
      if (isCreate) {
        const created = await api.adminPost('admin/marketing/catalog-rules/store', payload);
        if (created?.id) navigate(`/admin/marketing/promotions/catalog-rules/edit/${created.id}`);
      } else {
        await api.adminPut(`admin/marketing/catalog-rules/edit/${id}`, payload);
        setMessage(t('The catalog rule has been saved and prices reindexed.'));
      }
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the catalog rule.'));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div>
        <AdminPageHeader title={t('Loading catalog rule…')} backTo="/admin/marketing/promotions/catalog-rules" />
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
        title={isCreate ? t('Create catalog rule') : t('Edit catalog rule · {name}', { name: rule.name })}
        subtitle={t('Apply automatic price changes to matching products.')}
        backTo="/admin/marketing/promotions/catalog-rules"
        actions={
          <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
            <IconCheck size={15} /> {saving ? t('Saving…') : t('Save catalog rule')}
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
                <input className="admin-input" value={rule.name ?? ''} onChange={(event) => set('name', event.target.value)} placeholder={t('Summer catalog sale')} />
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
                <span className="admin-field-label">{t('Priority')}</span>
                <input className="admin-input" type="number" value={rule.sort_order ?? 0} onChange={(event) => set('sort_order', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('Start date')}</span>
                <input className="admin-input" type="date" value={(rule.starts_from ?? '').slice(0, 10)} onChange={(event) => set('starts_from', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">{t('End date')}</span>
                <input className="admin-input" type="date" value={(rule.ends_till ?? '').slice(0, 10)} onChange={(event) => set('ends_till', event.target.value)} />
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
                {(rule.conditions ?? []).length === 0 && <p className="admin-muted">{t('No conditions. The rule applies to every product.')}</p>}
                {(rule.conditions ?? []).map((condition: any, index: number) => (
                  <div className="admin-condition-row" key={index}>
                    <select className="admin-input" value={condition.attribute ?? ''} onChange={(event) => updateCondition(index, 'attribute', event.target.value)}>
                      {ATTRIBUTES.map((attribute) => (
                        <option key={attribute.value} value={attribute.value}>
                          {t(attribute.label)}
                        </option>
                      ))}
                    </select>
                    <select className="admin-input" value={condition.operator ?? '=='} onChange={(event) => updateCondition(index, 'operator', event.target.value)}>
                      {OPERATORS.map((operator) => (
                        <option key={operator.value} value={operator.value}>
                          {t(operator.label)}
                        </option>
                      ))}
                    </select>
                    <input className="admin-input" value={condition.value ?? ''} onChange={(event) => updateCondition(index, 'value', event.target.value)} placeholder={t('Value')} />
                    <button
                      type="button"
                      className="admin-icon-action danger"
                      onClick={() => set('conditions', (rule.conditions ?? []).filter((_: any, position: number) => position !== index))}
                    >
                      <IconTrash size={15} />
                    </button>
                  </div>
                ))}
              </div>
              <button
                type="button"
                className="admin-button"
                style={{ marginTop: 12 }}
                onClick={() => set('conditions', [...(rule.conditions ?? []), { attribute: 'product::price', operator: '>=', value: '' }])}
              >
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
              <label className="admin-check-row">
                <input type="checkbox" checked={!!rule.end_other_rules} onChange={(event) => set('end_other_rules', event.target.checked)} />
                <span>{t('Stop further rules from processing')}</span>
              </label>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
