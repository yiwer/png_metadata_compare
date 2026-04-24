# 站牌图片 JSON 元数据说明

## 概述

站牌渲染图片（PNG 格式）中嵌入了 JSON 格式的来源数据元数据，存储在 PNG 文件的 **iTXt** chunk 中。

- **chunk 关键字**：`StopPlateMetadata`
- **编码**：UTF-8
- **格式**：JSON（无缩进，紧凑格式）

## 提取方法

### 方法一：ExifTool（命令行）

```bash
exiftool -textualdata image.png
```

### 方法二：pngcheck

```bash
pngcheck -t image.png
```

### 方法三：Python

```python
import json
import struct

def read_stop_plate_metadata(png_path):
    with open(png_path, "rb") as f:
        data = f.read()

    pos = 8  # 跳过 PNG 签名
    while pos + 8 < len(data):
        length = struct.unpack(">I", data[pos:pos+4])[0]
        chunk_type = data[pos+4:pos+8].decode("ascii")

        if pos + 8 + length + 4 > len(data):
            break

        if chunk_type == "iTXt":
            chunk_data = data[pos+8:pos+8+length]
            null_idx = chunk_data.index(0)
            keyword = chunk_data[:null_idx].decode("latin-1")
            if keyword == "StopPlateMetadata":
                # keyword\0 + compression_flag(1) + compression_method(1) + language\0 + translated\0 + text
                text_start = null_idx + 1 + 1 + 1 + 1 + 1
                json_text = chunk_data[text_start:].decode("utf-8")
                return json.loads(json_text)

        pos += 4 + 4 + length + 4

    return None

# 使用示例
metadata = read_stop_plate_metadata("翻身地铁站_北_01_1050x1660_A.png")
print(json.dumps(metadata, ensure_ascii=False, indent=2))
```
## JSON 结构

### 顶层字段（StopPlateMetadata）

| 字段 | 类型 | 说明 |
|------|------|------|
| StopId | int? | 站点 ID |
| StopName | string | 中文站名 |
| StopEngName | string | 英文站名 |
| OriName | string | 原站名（站名变更前的名称） |
| RoadName | string | 所在道路 |
| DirectionOnRoad | string | 方位（东侧/西侧/南侧/北侧） |
| DistrictName | string | 所属行政区 |
| StreetCommitteeName | string | 所属街道 |
| QRCode | string | 二维码数据 |
| HasHints | bool | 该图片是否包含温馨提示区域 |
| Hints | string | 温馨提示内容（线路变更通知等） |
| IsGroupPrint | bool | 是否包含分站信息 |
| GroupItems | array | 分站信息列表，见下方 |
| IsBack | bool | false=正面，true=反面 |
| FrameSize | string | 站架尺寸，如 "600x900"（宽x高，毫米） |
| Lines | array | 线路信息列表，见下方 |
| RenderTime | string | 渲染时间，格式 "yyyy-MM-dd HH:mm:ss" |

### 分站信息（GroupItems 元素）

同名站点存在多个分站时提供。

| 字段 | 类型 | 说明 |
|------|------|------|
| SequenceNo | string | 分站序号（如 "①"） |
| LineNames | string | 该分站停靠的线路名称 |
| Distance | int | 与当前站的距离（米） |
| IsCurrent | bool | 是否为当前站点 |

### 线路信息（Lines 元素）

| 字段 | 类型 | 说明 |
|------|------|------|
| LineName | string | 线路名称（如 "B932"、"M375"、"1"） |
| Direction | string | 开往方向（终点站名） |
| FirstStopName | string | 线路起点站 |
| LastStopName | string | 线路终点站 |
| NextStop | string | 下一站名称（可能为 null） |
| CurrentStopSequence | int | 当前站在该线路中的站序号 |
| IsStarting | bool | 当前站是否为该线路起点站 |
| IsEnding | bool | 当前站是否为该线路终点站 |
| HeadBusCorpName | string | 运营企业 |
| TicketType | string | 票制（"一票制" 或 "分段收费"） |
| PriceDescription | string | 票价描述（如 "一票制 2元"、"分段收费 上车2元 全程6元"） |
| ServiceTimeDescription | string | 服务时间（如 "06:30-23:10"） |
| ScheduledServiceDescription | string | 发车时刻表（定时班车时提供） |
| LinePattern | string | 线路模式（"单边" 或 "双边"） |
| RouteStops | array | 途经站点列表，见下方 |

### 途经站点（RouteStops 元素）

| 字段 | 类型 | 说明 |
|------|------|------|
| Name | string | 站点名称 |
| Sequence | int | 站点在线路中的序号 |
| BuildingType | string | 附近设施图标类型，可能的值见下方 |
| RoadName | string | 该站点所在道路 |

**BuildingType 可能的值：**

| 值 | 含义 |
|----|------|
| 地铁 | 地铁换乘站 |
| 公交 | 公交换乘站 |
| 医院 | 医院 |
| 长途 | 长途客运站 |
| *(空/null)* | 无特殊设施标注 |

## 完整示例

```json
{
  "StopId": 1234,
  "StopName": "翻身地铁站",
  "StopEngName": "Fanshen Metro Station",
  "OriName": null,
  "RoadName": "创业一路",
  "DirectionOnRoad": "北侧",
  "DistrictName": "宝安区",
  "StreetCommitteeName": "新安",
  "QRCode": "https://example.com/stop/1234",
  "HasHints": false,
  "Hints": null,
  "IsGroupPrint": true,
  "GroupItems": [
    { "SequenceNo": "①", "LineNames": "M197", "Distance": 150, "IsCurrent": false },
    { "SequenceNo": "②", "LineNames": "B932,E15,M131...", "Distance": 0, "IsCurrent": true }
  ],
  "IsBack": false,
  "FrameSize": "1050x1660",
  "Lines": [
    {
      "LineName": "B932",
      "Direction": "福城万达广场公交总站",
      "FirstStopName": "福城万达广场公交总站",
      "LastStopName": "福城万达广场公交总站",
      "NextStop": "尚都花园",
      "CurrentStopSequence": 8,
      "IsStarting": false,
      "IsEnding": false,
      "HeadBusCorpName": "西部公汽",
      "TicketType": "一票制",
      "PriceDescription": "一票制 1元",
      "ServiceTimeDescription": "07:00-21:00",
      "ScheduledServiceDescription": null,
      "LinePattern": "双边",
      "RouteStops": [
        { "Name": "福城万达广场公交总站", "Sequence": 1, "BuildingType": null, "RoadName": "福城路" },
        { "Name": "上川路口", "Sequence": 2, "BuildingType": null, "RoadName": "福城路" },
        { "Name": "翻身地铁站", "Sequence": 8, "BuildingType": "地铁", "RoadName": "创业一路" },
        { "Name": "尚都花园", "Sequence": 9, "BuildingType": null, "RoadName": "创业一路" }
      ]
    }
  ],
  "RenderTime": "2026-04-02 18:40:16"
}
```

> 注：以上示例为演示结构，实际 RouteStops 包含该线路全部途经站点。

## 技术细节

- 元数据存储在 PNG iTXt chunk 中，位于 IEND chunk 之前
- 不影响图片的正常显示和使用
- 单条元数据体积通常为 2-10 KB
- 所有文本使用 UTF-8 编码，完整支持中文
