// frontend/src/lib/treeModel.test.ts
import { describe, it, expect } from 'vitest';
import { buildTree, buildMirrorRows, hasDiffDeep } from './treeModel';
import type { DiffNode, JsonValue } from './types';

describe('buildTree', () => {
  it('builds an object-root with leaf children', () => {
    const tree = buildTree({ StopName: '翻身地铁站', StopId: 1234 });
    expect(tree.kind).toBe('group');
    expect(tree.variant).toBe('object-root');
    expect(tree.children).toHaveLength(2);

    const stopName = tree.children[0];
    expect(stopName).toMatchObject({
      kind: 'leaf',
      path: 'StopName',
      label: '中文站名',
      value: '翻身地铁站',
    });
    const stopId = tree.children[1];
    expect(stopId).toMatchObject({
      kind: 'leaf',
      path: 'StopId',
      label: '站点编号',
      value: '1234',
    });
  });

  it('marks unknown fields with isUnknown', () => {
    const tree = buildTree({ Foo: 'bar' });
    expect(tree.children[0]).toMatchObject({ label: 'Foo', isUnknown: true });
  });

  it('renders null/empty/booleans through formatValue', () => {
    const tree = buildTree({ HasHints: false, Hints: null, OriName: '' });
    expect((tree.children[0] as any).value).toBe('否');
    expect((tree.children[1] as any).value).toBe('—');
    expect((tree.children[2] as any).value).toBe('—');
  });

  it('builds an array group with array-item children using arrayItemLabel', () => {
    const tree = buildTree({
      Lines: [
        { LineName: 'B932', Direction: '终点A' },
        { LineName: 'M375', Direction: '终点B' },
      ],
    } as JsonValue);

    const linesGroup = tree.children[0];
    expect(linesGroup.kind).toBe('group');
    expect((linesGroup as any).variant).toBe('array');
    expect((linesGroup as any).count).toBe(2);
    expect((linesGroup as any).label).toBe('停靠线路');
    expect((linesGroup as any).defaultOpen).toBe(false);

    const item0 = (linesGroup as any).children[0];
    expect(item0.variant).toBe('array-item');
    expect(item0.label).toBe('线路 1 · B932');
    expect(item0.path).toBe('Lines[0]');
    expect(item0.children).toHaveLength(2);
    expect(item0.children[0]).toMatchObject({
      path: 'Lines[0].LineName',
      label: '线路名称',
      value: 'B932',
    });
  });

  it('handles deeply nested arrays (RouteStops)', () => {
    const tree = buildTree({
      Lines: [
        {
          LineName: 'B932',
          RouteStops: [
            { Name: '上川路口', Sequence: 2, BuildingType: null, RoadName: '福城路' },
          ],
        },
      ],
    } as JsonValue);

    const lines = tree.children[0] as any;
    const line0 = lines.children[0];
    const routeStops = line0.children.find((c: any) => c.path === 'Lines[0].RouteStops');
    expect(routeStops.variant).toBe('array');
    expect(routeStops.children[0].label).toBe('#2 · 上川路口');
    const routeStop0 = routeStops.children[0];
    const buildingType = routeStop0.children.find((c: any) => c.path.endsWith('.BuildingType'));
    expect(buildingType.value).toBe('—');
  });

  it('uses default-open for object groups but default-closed for arrays', () => {
    const tree = buildTree({
      RoadName: '创业一路',
      Lines: [{ LineName: 'B932' }],
      GroupItems: [{ SequenceNo: '①' }],
    } as JsonValue);
    const lines = tree.children.find((c) => c.path === 'Lines') as any;
    const groups = tree.children.find((c) => c.path === 'GroupItems') as any;
    expect(lines.defaultOpen).toBe(false);
    expect(groups.defaultOpen).toBe(false);
  });
});

describe('buildMirrorRows', () => {
  const noDiff: DiffNode = {
    path: '', status: 'unchanged', left_value: null, right_value: null, summary: '', children: [],
  };

  it('renders leaves with both values when keys match', () => {
    const rows = buildMirrorRows({ StopName: '翻身' }, { StopName: '翻身' }, noDiff);
    expect(rows[0].children![0]).toMatchObject({
      kind: 'leaf', path: 'StopName', label: '中文站名',
      leftValue: '翻身', rightValue: '翻身', status: 'unchanged',
    });
  });

  it('renders missing side as em dash, not as null', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'Hints', status: 'removed', left_value: '"x"', right_value: null, summary: '', children: [] }],
    };
    const rows = buildMirrorRows({ Hints: 'x' }, {}, diff);
    const leaf = rows[0].children!.find((c) => c.path === 'Hints')!;
    expect(leaf).toMatchObject({
      path: 'Hints', leftValue: 'x', rightValue: '—', status: 'removed',
    });
  });

  it('renders schema-known field absent on left as em-dash on left', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'Hints', status: 'added', left_value: null, right_value: '"y"', summary: '', children: [] }],
    };
    const rows = buildMirrorRows({}, { Hints: 'y' }, diff);
    const leaf = rows[0].children!.find((c) => c.path === 'Hints')!;
    expect(leaf).toMatchObject({
      path: 'Hints', leftValue: '—', rightValue: 'y', status: 'added',
    });
  });

  it('marks modified leaves with both values present and status modified', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'FrameSize', status: 'modified', left_value: '"1050x1660"', right_value: '"1200x1800"', summary: '', children: [] }],
    };
    const rows = buildMirrorRows(
      { FrameSize: '1050x1660' },
      { FrameSize: '1200x1800' },
      diff,
    );
    const leaf = rows[0].children![0];
    expect(leaf).toMatchObject({
      path: 'FrameSize', leftValue: '1050x1660', rightValue: '1200x1800', status: 'modified',
    });
  });

  it('looks up nested RouteStop status using the backend keyed path', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [
        {
          path: 'Lines', status: 'modified', left_value: null, right_value: null, summary: '',
          children: [
            {
              path: 'Lines[718|宝安壹方中心]', status: 'modified', left_value: null, right_value: null, summary: '',
              children: [
                {
                  path: 'Lines[718|宝安壹方中心].RouteStops', status: 'modified', left_value: null, right_value: null, summary: '',
                  children: [
                    {
                      path: 'Lines[718|宝安壹方中心].RouteStops[西乡恒生医院]', status: 'modified', left_value: null, right_value: null, summary: '',
                      children: [
                        {
                          path: 'Lines[718|宝安壹方中心].RouteStops[西乡恒生医院].BuildingType',
                          status: 'modified', left_value: '"Hospital"', right_value: '"医院"', summary: '', children: [],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    };
    const left = {
      Lines: [{
        LineName: '718', Direction: '宝安壹方中心',
        RouteStops: [{ Name: '西乡恒生医院', BuildingType: 'Hospital', RoadName: '银田路' }],
      }],
    };
    const right = {
      Lines: [{
        LineName: '718', Direction: '宝安壹方中心',
        RouteStops: [{ Name: '西乡恒生医院', BuildingType: '医院', RoadName: '银田路' }],
      }],
    };
    const rows = buildMirrorRows(left as JsonValue, right as JsonValue, diff);

    function findByPath(rs: any[], target: string): any | undefined {
      for (const r of rs) {
        if (r.path === target) return r;
        if (r.children) {
          const found = findByPath(r.children, target);
          if (found) return found;
        }
      }
      return undefined;
    }

    const buildingTypeLeaf = findByPath(
      rows,
      'Lines[718|宝安壹方中心].RouteStops[西乡恒生医院].BuildingType',
    );
    expect(buildingTypeLeaf).toBeDefined();
    expect(buildingTypeLeaf.status).toBe('modified');
    // Through formatValue (BuildingType formatter): unknown enum passes through, mapped enum translates.
    expect(buildingTypeLeaf.leftValue).toBe('Hospital');
    expect(buildingTypeLeaf.rightValue).toBe('医院');
  });

  it('sorts RouteStops by Sequence ascending in display order', () => {
    const left = {
      Lines: [{
        LineName: '718', Direction: 'X',
        RouteStops: [
          { Name: 'C', Sequence: 2 },
          { Name: 'A', Sequence: 0 },
          { Name: 'B', Sequence: 1 },
        ],
      }],
    };
    const right = { Lines: [{ LineName: '718', Direction: 'X', RouteStops: [] }] };
    const noDiffRoot: DiffNode = {
      path: '', status: 'unchanged', left_value: null, right_value: null, summary: '', children: [],
    };
    const rows = buildMirrorRows(left as JsonValue, right as JsonValue, noDiffRoot);

    function findByPath(rs: any[], target: string): any | undefined {
      for (const r of rs) {
        if (r.path === target) return r;
        if (r.children) {
          const found = findByPath(r.children, target);
          if (found) return found;
        }
      }
      return undefined;
    }
    const routeStops = findByPath(rows, 'Lines[718|X].RouteStops');
    expect(routeStops).toBeDefined();
    const orderedNames = routeStops.children.map((c: any) => c.path.split('[').pop()?.replace(']', ''));
    expect(orderedNames).toEqual(['A', 'B', 'C']);
  });

  it('aligns Lines by business key and pads the missing side', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [
        {
          path: 'Lines', status: 'modified', left_value: null, right_value: null, summary: '', children: [
            { path: 'Lines[M375]', status: 'added', left_value: null, right_value: null, summary: '', children: [] },
          ],
        },
      ],
    };
    const rows = buildMirrorRows(
      { Lines: [{ LineName: 'B932' }] },
      { Lines: [{ LineName: 'B932' }, { LineName: 'M375' }] },
      diff,
    );
    const lines = rows[0].children![0];
    expect(lines.kind).toBe('group');
    expect(lines.children).toHaveLength(2);
    const newLine = lines.children!.find((c) => c.path === 'Lines[M375]')!;
    expect(newLine).toBeDefined();
    expect(newLine.status).toBe('added');
    // The missing left item still produces a row whose leaves all render as a blank
    // placeholder on left (em-dash, possibly with extra context per field formatter).
    const itemLeaves = newLine.children!;
    expect(itemLeaves.every((l) => l.leftValue !== null && l.leftValue.startsWith('—'))).toBe(true);
  });
});

describe('hasDiffDeep', () => {
  it('returns true when any descendant is non-unchanged', () => {
    const row = {
      kind: 'group' as const, path: '', label: 'r', leftValue: null, rightValue: null,
      status: 'unchanged' as const, isUnknown: false, defaultOpen: true,
      children: [
        {
          kind: 'leaf' as const, path: 'a', label: 'a', leftValue: '1', rightValue: '2',
          status: 'modified' as const, isUnknown: false, defaultOpen: true,
        },
      ],
    };
    expect(hasDiffDeep(row)).toBe(true);
  });
  it('returns false when whole subtree unchanged', () => {
    const row = {
      kind: 'leaf' as const, path: 'a', label: 'a', leftValue: '1', rightValue: '1',
      status: 'unchanged' as const, isUnknown: false, defaultOpen: true,
    };
    expect(hasDiffDeep(row)).toBe(false);
  });
});
