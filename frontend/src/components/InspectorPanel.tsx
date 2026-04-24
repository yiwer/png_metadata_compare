import type { AnalysisTab, JsonValue, PairInspection, SideInspection } from '../lib/types';

export function InspectorPanel({
  inspection,
  singleSideInspection,
  activePath,
  activeTab,
}: {
  inspection: PairInspection | null;
  singleSideInspection: SideInspection | null;
  activePath: string | null;
  activeTab: AnalysisTab;
}) {
  const diffNode = inspection && activePath ? findDiffNode(inspection.diff_root, activePath) : null;
  const leftMetadata =
    inspection?.left.metadata ??
    (singleSideInspection?.side === 'left' ? singleSideInspection.metadata : null);
  const rightMetadata =
    inspection?.right.metadata ??
    (singleSideInspection?.side === 'right' ? singleSideInspection.metadata : null);
  const selectedMetadata =
    activeTab === 'left_metadata'
      ? getJsonValueAtPath(leftMetadata, activePath)
      : activeTab === 'right_metadata'
        ? getJsonValueAtPath(rightMetadata, activePath)
        : undefined;

  return (
    <div className="inspector-stack">
      <dl className="inspector-row">
        <dt>Selected Path</dt>
        <dd>{activePath ?? 'None'}</dd>
      </dl>
      <dl className="inspector-row">
        <dt>View</dt>
        <dd>{labelForTab(activeTab)}</dd>
      </dl>

      {activeTab === 'diff' ? (
        <>
          <dl className="inspector-row">
            <dt>Status</dt>
            <dd>{diffNode?.status ?? (inspection ? 'Select a diff node' : 'No paired result')}</dd>
          </dl>
          <dl className="inspector-row">
            <dt>Left Value</dt>
            <dd>{diffNode?.left_value ?? '<none>'}</dd>
          </dl>
          <dl className="inspector-row">
            <dt>Right Value</dt>
            <dd>{diffNode?.right_value ?? '<none>'}</dd>
          </dl>
        </>
      ) : null}

      {(activeTab === 'left_metadata' || activeTab === 'right_metadata') && selectedMetadata !== undefined ? (
        <>
          <dl className="inspector-row">
            <dt>Value Type</dt>
            <dd>{typeForJsonValue(selectedMetadata)}</dd>
          </dl>
          <dl className="inspector-row">
            <dt>Value</dt>
            <dd className="inspector-row__code">{formatJsonValue(selectedMetadata)}</dd>
          </dl>
        </>
      ) : null}

      {activeTab === 'raw_json' ? (
        <>
          <dl className="inspector-row">
            <dt>Left JSON</dt>
            <dd>{inspection?.left.raw_json ?? (singleSideInspection?.side === 'left' ? singleSideInspection.raw_json : null) ? 'Available' : 'Missing'}</dd>
          </dl>
          <dl className="inspector-row">
            <dt>Right JSON</dt>
            <dd>{inspection?.right.raw_json ?? (singleSideInspection?.side === 'right' ? singleSideInspection.raw_json : null) ? 'Available' : 'Missing'}</dd>
          </dl>
        </>
      ) : null}

      {activeTab === 'images' ? (
        <>
          <dl className="inspector-row">
            <dt>Left Image</dt>
            <dd>{inspection?.left.file_name ?? (singleSideInspection?.side === 'left' ? singleSideInspection.file_name : 'Missing')}</dd>
          </dl>
          <dl className="inspector-row">
            <dt>Right Image</dt>
            <dd>{inspection?.right.file_name ?? (singleSideInspection?.side === 'right' ? singleSideInspection.file_name : 'Missing')}</dd>
          </dl>
        </>
      ) : null}
    </div>
  );
}

function labelForTab(tab: AnalysisTab) {
  switch (tab) {
    case 'diff':
      return 'Diff';
    case 'left_metadata':
      return 'Left Metadata';
    case 'right_metadata':
      return 'Right Metadata';
    case 'raw_json':
      return 'Raw JSON';
    case 'images':
      return 'Images';
  }
}

function findDiffNode(node: PairInspection['diff_root'], path: string): PairInspection['diff_root'] | null {
  if (node.path === path) {
    return node;
  }

  for (const child of node.children) {
    const found = findDiffNode(child, path);
    if (found) {
      return found;
    }
  }

  return null;
}

function getJsonValueAtPath(source: JsonValue | null, path: string | null) {
  if (source === null) {
    return undefined;
  }

  if (!path) {
    return source;
  }

  let current: JsonValue | undefined = source;
  const tokens = path.match(/([^[.\]]+)|\[(\d+)\]/g) ?? [];

  for (const token of tokens) {
    if (token.startsWith('[')) {
      const index = Number(token.slice(1, -1));
      current = Array.isArray(current) ? current[index] : undefined;
    } else {
      current =
        current && typeof current === 'object' && !Array.isArray(current)
          ? (current as Record<string, JsonValue>)[token]
          : undefined;
    }

    if (current === undefined) {
      return undefined;
    }
  }

  return current;
}

function typeForJsonValue(value: JsonValue) {
  if (Array.isArray(value)) {
    return 'array';
  }

  if (value === null) {
    return 'null';
  }

  return typeof value === 'object' ? 'object' : typeof value;
}

function formatJsonValue(value: JsonValue) {
  if (typeof value === 'string') {
    return value;
  }

  if (value === null) {
    return 'null';
  }

  return typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value);
}
