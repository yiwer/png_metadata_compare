import { describe, it, expect } from 'vitest';
import {
  fieldLabel,
  fieldDef,
  formatValue,
  arrayItemLabel,
  arrayKeyFn,
  isKnownField,
} from './labels';

describe('arrayKeyFn', () => {
  it('keys Lines by LineName only — Direction changes must not break matching', () => {
    const keyFn = arrayKeyFn('Lines')!;
    expect(keyFn({ LineName: 'M208', Direction: '旧终点站' })).toBe('M208');
    expect(keyFn({ LineName: 'M208', Direction: '新终点站' })).toBe('M208');
    expect(keyFn({ LineName: 'M208' })).toBe('M208');
    expect(keyFn({ Direction: '无名线路' })).toBeNull();
  });
});

describe('fieldLabel', () => {
  it('returns Chinese label for top-level fields', () => {
    expect(fieldLabel('StopName')).toBe('中文站名');
    expect(fieldLabel('DirectionOnRoad')).toBe('在道路的方位');
    expect(fieldLabel('FrameSize')).toBe('站架尺寸');
    expect(fieldLabel('RenderTime')).toBe('渲染时间');
  });

  it('normalizes array-element paths via [*]', () => {
    expect(fieldLabel('Lines[3].LineName')).toBe('线路名称');
    expect(fieldLabel('Lines[0].RouteStops[5].Name')).toBe('站点名称');
    expect(fieldLabel('GroupItems[1].SequenceNo')).toBe('分站序号');
  });

  it('falls back to the raw key for unknown paths', () => {
    expect(fieldLabel('UnknownField')).toBe('UnknownField');
    expect(fieldLabel('Lines[0].UnknownNested')).toBe('UnknownNested');
  });
});

describe('isKnownField', () => {
  it('flags known and unknown', () => {
    expect(isKnownField('StopName')).toBe(true);
    expect(isKnownField('Lines[*].LineName')).toBe(true);
    expect(isKnownField('Lines[2].LineName')).toBe(true);
    expect(isKnownField('UnknownField')).toBe(false);
  });
});

describe('formatValue', () => {
  it('formats null → em dash', () => {
    expect(formatValue('OriName', null)).toBe('—');
    expect(formatValue('Lines[0].NextStop', null)).toBe('—（已到终点）');
    expect(formatValue('Lines[0].ScheduledServiceDescription', null)).toBe('—（非定时班车）');
  });

  it('formats empty string the same as null → "—"', () => {
    expect(formatValue('StopName', '')).toBe('—');
    expect(formatValue('StopName', null)).toBe('—');
    expect(formatValue('StopName', undefined)).toBe('—');
  });

  it('renders booleans contextually', () => {
    expect(formatValue('HasHints', true)).toBe('是');
    expect(formatValue('HasHints', false)).toBe('否');
    expect(formatValue('IsBack', true)).toBe('反面');
    expect(formatValue('IsBack', false)).toBe('正面');
    expect(formatValue('GroupItems[0].IsCurrent', true)).toBe('是（当前）');
    expect(formatValue('GroupItems[0].IsCurrent', false)).toBe('否');
  });

  it('appends unit to Distance', () => {
    expect(formatValue('GroupItems[0].Distance', 150)).toBe('150 米');
  });

  it('translates BuildingType enum', () => {
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '地铁')).toBe('地铁换乘');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '公交')).toBe('公交换乘');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '医院')).toBe('医院');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '长途')).toBe('长途客运');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', null)).toBe('—');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '')).toBe('—');
  });

  it('passes through plain string / number / unknown booleans', () => {
    expect(formatValue('StopName', '翻身地铁站')).toBe('翻身地铁站');
    expect(formatValue('StopId', 1234)).toBe('1234');
    expect(formatValue('Lines[0].IsStarting', true)).toBe('是');
    expect(formatValue('Lines[0].IsStarting', false)).toBe('否');
  });
});

describe('arrayItemLabel', () => {
  it('uses SequenceNo for GroupItems', () => {
    expect(arrayItemLabel('GroupItems', 0, { SequenceNo: '①' })).toBe('分站 ①');
    expect(arrayItemLabel('GroupItems', 1, { SequenceNo: '②' })).toBe('分站 ②');
    expect(arrayItemLabel('GroupItems', 0, {})).toBe('分站 1');
  });

  it('uses LineName for Lines', () => {
    expect(arrayItemLabel('Lines', 0, { LineName: 'B932' })).toBe('线路 1 · B932');
    expect(arrayItemLabel('Lines', 2, { LineName: 'M375' })).toBe('线路 3 · M375');
    expect(arrayItemLabel('Lines', 0, {})).toBe('线路 1 · ?');
  });

  it('uses Sequence + Name for RouteStops', () => {
    expect(arrayItemLabel('Lines[*].RouteStops', 2, { Sequence: 3, Name: '翻身地铁站' }))
      .toBe('#3 · 翻身地铁站');
    expect(arrayItemLabel('Lines[*].RouteStops', 0, {})).toBe('#1 · ?');
  });

  it('falls back to "项 N" for unknown arrays', () => {
    expect(arrayItemLabel('UnknownArray', 4, {})).toBe('项 5');
  });
});

describe('fieldDef', () => {
  it('returns undefined for unknown path', () => {
    expect(fieldDef('NotAField')).toBeUndefined();
  });
  it('returns the def for known path', () => {
    expect(fieldDef('StopName')).toBeDefined();
    expect(fieldDef('Lines[2].LineName')).toBeDefined();
  });
});
