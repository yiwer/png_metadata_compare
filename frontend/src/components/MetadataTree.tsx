import type { JsonValue } from '../lib/types';

export function MetadataTree({
  value,
  prefix = '',
  activePath,
  onSelect,
}: {
  value: JsonValue;
  prefix?: string;
  activePath: string | null;
  onSelect(path: string): void;
}) {
  if (Array.isArray(value)) {
    return (
      <div className="json-tree">
        {value.map((entry, index) => {
          const path = `${prefix}[${index}]`;
          return (
            <MetadataTree
              key={path}
              value={entry}
              prefix={path}
              activePath={activePath}
              onSelect={onSelect}
            />
          );
        })}
      </div>
    );
  }

  if (value && typeof value === 'object') {
    return (
      <div className="json-tree">
        {Object.entries(value).map(([key, child]) => {
          const path = prefix ? `${prefix}.${key}` : key;
          return (
            <div key={path} className="json-tree__branch">
              <button
                type="button"
                className={activePath === path ? 'tree__button tree__button--active' : 'tree__button'}
                onClick={() => onSelect(path)}
              >
                <span>{path}</span>
                <small className="tree__meta">{describeJsonValue(child)}</small>
              </button>
              <MetadataTree value={child} prefix={path} activePath={activePath} onSelect={onSelect} />
            </div>
          );
        })}
      </div>
    );
  }

  const label = prefix || 'value';
  return (
    <button
      type="button"
      className={activePath === label ? 'tree__button tree__button--active' : 'tree__button'}
      onClick={() => onSelect(label)}
    >
      <span>
        {label}: {String(value)}
      </span>
      <small className="tree__meta">{describeJsonValue(value)}</small>
    </button>
  );
}

function describeJsonValue(value: JsonValue) {
  if (Array.isArray(value)) {
    return `${value.length} items`;
  }

  if (value && typeof value === 'object') {
    return `${Object.keys(value).length} fields`;
  }

  return typeof value;
}
