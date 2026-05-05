export interface FieldDef {
  label: string;
  format?: (value: unknown) => string;
}

const yesNo = (v: unknown): string => (v === true ? '是' : v === false ? '否' : String(v));

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
  IsBack: { label: '站牌朝向', format: (v) => (v === true ? '反面' : v === false ? '正面' : String(v)) },
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

/** 把运行时路径（含数字下标）归一化为 schema key（用 `[*]` 占位）。 */
function normalizePath(path: string): string {
  return path.replace(/\[\d+\]/g, '[*]');
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

  if (value === null || value === undefined) return '—';
  if (value === '') return '(空)';
  if (typeof value === 'boolean') return value ? '是' : '否';
  return String(value);
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
