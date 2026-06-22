import { useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { useImageTransform } from '../lib/useImageTransform';

export function ImagePane({ path, name, transform }: { path: string; name: string; transform: string }) {
  const url = convertFileSrc(path);
  const [broken, setBroken] = useState(false);
  return (
    <div className="image-pane">
      <div className="image-pane__viewport">
        {broken ? (
          <div className="image-pane__broken">无法加载图片</div>
        ) : (
          <img className="image-pane__img" src={url} alt={name} draggable={false}
            style={{ transform, transformOrigin: 'center center' }}
            onError={() => setBroken(true)} />
        )}
      </div>
      <div className="image-pane__bar">
        <span className="image-pane__name">{name}</span>
        <button type="button" className="detail-head__btn" onClick={() => void openPath(path)}>打开原文件 ↗</button>
      </div>
    </div>
  );
}

function ZoomToolbar({ t, summary }: { t: ReturnType<typeof useImageTransform>; summary: string }) {
  return (
    <div className="image-split__toolbar">
      <button type="button" className="controlbar__btn" onClick={t.zoomOut}>−</button>
      <span className="controlbar__summary" style={{ minWidth: 56, textAlign: 'center' }}>{Math.round(t.zoom * 100)}%</span>
      <button type="button" className="controlbar__btn" onClick={t.zoomIn}>＋</button>
      <button type="button" className="controlbar__btn" onClick={t.reset}>重置</button>
      <span className="controlbar__spacer" />
      <span className="controlbar__summary">{summary}</span>
    </div>
  );
}

export function ImageSplit({ leftPath, rightPath, leftName, rightName }: { leftPath: string; rightPath: string; leftName: string; rightName: string; }) {
  const t = useImageTransform();
  return (
    <div className="image-split">
      <ZoomToolbar t={t} summary="滚轮缩放 · 拖拽平移（左右同步）" />
      <div
        className={`image-split__panes${t.dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={t.onWheel} onMouseDown={t.onMouseDown} onMouseMove={t.onMouseMove}
        onMouseUp={t.endDrag} onMouseLeave={t.endDrag}
      >
        <ImagePane key={leftPath} path={leftPath} name={leftName} transform={t.transform} />
        <ImagePane key={rightPath} path={rightPath} name={rightName} transform={t.transform} />
      </div>
    </div>
  );
}

export function SoloImage({ path, name }: { path: string; name: string }) {
  const t = useImageTransform();
  return (
    <div className="image-split">
      <ZoomToolbar t={t} summary="滚轮缩放 · 拖拽平移" />
      <div
        className={`image-split__panes image-split__panes--solo${t.dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={t.onWheel} onMouseDown={t.onMouseDown} onMouseMove={t.onMouseMove}
        onMouseUp={t.endDrag} onMouseLeave={t.endDrag}
      >
        <ImagePane key={path} path={path} name={name} transform={t.transform} />
      </div>
    </div>
  );
}
