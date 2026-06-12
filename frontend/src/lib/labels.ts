export interface FieldDef {
  label: string;
  format?: (value: unknown) => string;
}

const yesNo = (v: unknown): string => {
  if (v === true) return '是';
  if (v === false) return '否';
  if (v === null || v === undefined || v === '') return '—';
  return String(v);
};

const BUILDING_TYPE: Record<string, string> = {
  地铁: '地铁换乘',
  公交: '公交换乘',
  医院: '医院',
  长途: '长途客运',
};

export const FIELD_DEFS: Record<string, FieldDef> = {
  // 顶层
  StopId: { label: '站点编号' },
  StopName: { label: '中文站名' },
  StopEngName: { label: '英文站名' },
  OriName: { label: '原站名' },
  RoadName: { label: '所在道路' },
  DirectionOnRoad: { label: '在道路的方位' },
  DistrictName: { label: '所属行政区' },
  StreetCommitteeName: { label: '所属街道' },
  QRCode: { label: '二维码地址' },
  HasHints: { label: '含温馨提示', format: yesNo },
  Hints: { label: '温馨提示内容' },
  IsGroupPrint: { label: '含分站信息', format: yesNo },
  GroupItems: { label: '分站列表' },
  IsBack: {
    label: '站牌朝向',
    format: (v) => {
      if (v === true) return '反面';
      if (v === false) return '正面';
      if (v === null || v === undefined || v === '') return '—';
      return String(v);
    },
  },
  FrameSize: { label: '站架尺寸' },
  Lines: { label: '停靠线路' },
  RenderTime: { label: '渲染时间' },

  // GroupItems 元素
  'GroupItems[*].SequenceNo': { label: '分站序号' },
  'GroupItems[*].LineNames': { label: '该分站线路' },
  'GroupItems[*].Distance': {
    label: '距当前站',
    format: (v) => (typeof v === 'number' ? `${v} 米` : String(v)),
  },
  'GroupItems[*].IsCurrent': {
    label: '是否当前站',
    format: (v) => (v === true ? '是（当前）' : v === false ? '否' : String(v)),
  },

  // Lines 元素
  'Lines[*].LineName': { label: '线路名称' },
  'Lines[*].Direction': { label: '开往方向' },
  'Lines[*].FirstStopName': { label: '线路起点站' },
  'Lines[*].LastStopName': { label: '线路终点站' },
  'Lines[*].NextStop': {
    label: '下一站',
    format: (v) => (v === null || v === undefined ? '—（已到终点）' : String(v)),
  },
  'Lines[*].CurrentStopSequence': { label: '当前站站序' },
  'Lines[*].IsStarting': { label: '当前站是起点', format: yesNo },
  'Lines[*].IsEnding': { label: '当前站是终点', format: yesNo },
  'Lines[*].HeadBusCorpName': { label: '运营企业' },
  'Lines[*].TicketType': { label: '票制类型' },
  'Lines[*].PriceDescription': { label: '票价描述' },
  'Lines[*].ServiceTimeDescription': { label: '服务时间' },
  'Lines[*].ScheduledServiceDescription': {
    label: '发车时刻表',
    format: (v) => (v === null || v === undefined ? '—（非定时班车）' : String(v)),
  },
  'Lines[*].LinePattern': { label: '线路模式' },
  'Lines[*].RouteStops': { label: '途经站点' },

  // RouteStops 元素
  'Lines[*].RouteStops[*].Name': { label: '站点名称' },
  'Lines[*].RouteStops[*].Sequence': { label: '站点序号' },
  'Lines[*].RouteStops[*].BuildingType': {
    label: '附近设施',
    format: (v) => {
      if (v === null || v === undefined || v === '') return '—';
      const s = String(v);
      return BUILDING_TYPE[s] ?? s;
    },
  },
  'Lines[*].RouteStops[*].RoadName': { label: '所在道路' },
};

/** 把运行时路径（含数字下标 / 业务键 / 重复后缀）归一化为 schema key（用 `[*]` 占位）。 */
function normalizePath(path: string): string {
  return path.replace(/\[[^\]]*\]/g, '[*]');
}

/** 取路径最后一段作为最后兜底标签（如 "UnknownNested"）。 */
function tailKey(path: string): string {
  const lastDot = path.lastIndexOf('.');
  return lastDot === -1 ? path : path.slice(lastDot + 1);
}

export function fieldDef(path: string): FieldDef | undefined {
  return FIELD_DEFS[normalizePath(path)];
}

export function fieldLabel(path: string): string {
  return fieldDef(path)?.label ?? tailKey(path);
}

export function isKnownField(path: string): boolean {
  return FIELD_DEFS[normalizePath(path)] !== undefined;
}

export function formatValue(path: string, value: unknown): string {
  const def = fieldDef(path);
  if (def?.format) return def.format(value);

  if (isBlank(value)) return '—';
  if (typeof value === 'boolean') return value ? '是' : '否';
  return String(value);
}

/** 空字符串、null、undefined 视作同等"无值"。 */
export function isBlank(value: unknown): boolean {
  return value === null || value === undefined || value === '';
}

/**
 * 根据 FIELD_DEFS 推导每个 normalized 父路径下声明过的子字段名。
 * 例如父路径 "Lines[*].RouteStops[*]" → ["Name", "Sequence", "BuildingType", "RoadName"]。
 */
const SCHEMA_CHILDREN: ReadonlyMap<string, readonly string[]> = (() => {
  const acc = new Map<string, Set<string>>();
  for (const key of Object.keys(FIELD_DEFS)) {
    const lastDot = key.lastIndexOf('.');
    const parent = lastDot === -1 ? '' : key.slice(0, lastDot);
    const child = lastDot === -1 ? key : key.slice(lastDot + 1);
    let bucket = acc.get(parent);
    if (!bucket) acc.set(parent, (bucket = new Set()));
    bucket.add(child);
  }
  const out = new Map<string, readonly string[]>();
  for (const [k, v] of acc) out.set(k, Array.from(v));
  return out;
})();

export function schemaChildKeys(parentPath: string): readonly string[] {
  return SCHEMA_CHILDREN.get(normalizePath(parentPath)) ?? [];
}

/**
 * 业务键计算函数（与后端 src/diff.rs#business_key_for_path 对齐）。
 * 仅用于路径段，不参与匹配本身；匹配在 treeModel.ts 中按消费式做。
 * 返回 null 表示该项缺业务键 / 不参与 keyed 匹配。
 */
export type ArrayKeyFn = (item: unknown) => string | null;

export function arrayKeyFn(arrayPath: string): ArrayKeyFn | null {
  switch (normalizePath(arrayPath)) {
    case 'Lines':
      // 只按线路名匹配（与后端一致）：开往方向是会被改动的属性（如终点站更名），
      // 进键会让同名线路各自落单成 仅左/仅右；同名重复项走消费式 #N 配对。
      return (item) => {
        const o = (item ?? {}) as Record<string, unknown>;
        return typeof o.LineName === 'string' ? o.LineName : null;
      };
    case 'GroupItems':
      return (item) => {
        const o = (item ?? {}) as Record<string, unknown>;
        return typeof o.SequenceNo === 'string' ? o.SequenceNo : null;
      };
    case 'Lines[*].RouteStops':
      return (item) => {
        const o = (item ?? {}) as Record<string, unknown>;
        return typeof o.Name === 'string' ? o.Name : null;
      };
    default:
      return null;
  }
}

export function arrayItemLabel(arrayPath: string, index: number, item: unknown): string {
  const norm = normalizePath(arrayPath);
  const obj = (item ?? {}) as Record<string, unknown>;
  switch (norm) {
    case 'GroupItems':
      return `分站 ${obj.SequenceNo ?? index + 1}`;
    case 'Lines':
      return `线路 ${index + 1} · ${obj.LineName ?? '?'}`;
    case 'Lines[*].RouteStops':
      return `#${obj.Sequence ?? index + 1} · ${obj.Name ?? '?'}`;
    default:
      return `项 ${index + 1}`;
  }
}
