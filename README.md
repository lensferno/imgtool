# imgtool

简单的命令行图片压缩和转换工具，基于 [libcaesium](https://github.com/Lymphatus/libcaesium)

* **JPEG、PNG、GIF、TIFF、WebP** 格式的压缩与转换
* 可批量处理文件夹中的所有图片
* 支持有损/无损压缩模式
* 支持按比例缩放、按边长缩放、固定尺寸等多种尺寸调整方式
* 可保留或移除图片元数据
* 可选择在输出文件大于源文件时跳过生成

~~没有详细测试过所有参数，所以不排除会出现BUG的情况~~

---

## 基本用法

```bash
imgtool [FLAGS] [OPTIONS] --input <input> --output <output>
```

* **`<input>`**：必选参数，输入文件或目录路径。
* **`<output>`**：必选参数，输出文件或目录路径。
* 当`--input`为文件时，`--output`如果存在且为文件夹，则输出到该文件夹，否则作为文件输出；
* 当`--input`为文件夹时，`--output`必须为文件夹

---

## 参数说明

### 通用标志 (FLAGS)

| 参数                    | 描述                  |
| --------------------- | ------------------- |
| `--continue-on-error` | 处理文件出错时继续执行后续任务     |
| `--delete-origin`     | 处理完成后删除源文件          |
| `-h`, `--help`        | 显示帮助信息              |
| `--keep-metadata`     | 保留压缩图片的元数据          |
| `--lossless`          | 使用无损压缩（质量可能仍会有下降） |
| `--skip-if-bigger`    | 如果压缩后文件大于原文件则跳过生成   |
| `-V`, `--version`     | 显示版本信息              |

---

### 输入/输出选项

| 参数                             | 描述                |
| ------------------------------ |-------------------|
| `-i, --input <input>`          | 输入文件或目录（必选）       |
| `-o, --output <output>`        | 输出文件或目录，默认与输入相同   |
| `-p, --prefix <prefix>`        | 输出文件名前缀，默认无前缀     |
| `-t, --target-format <format>` | 输出图片格式，不指定则与源文件相同 |

---

### 压缩参数

每种格式支持不同的压缩参数，可通过 `--xxx-params key=value,...` 设置：

#### JPEG

```bash
--jpeg-params quality=80,chroma_subsampling=auto,progressive=true
```

* **quality**：图片质量 (0-100)
* **chroma\_subsampling**：`cs444` / `cs422` / `cs420` / `cs411` / `auto`（默认）
* **progressive**：渐进式 JPEG (`true`/`false`，默认 `true`)

#### PNG

```bash
--png-params quality=80,force_zopfli=false,optimization_level=2
```

* **quality**：质量 (0-100)
* **force\_zopfli**：启用 Zopfli 压缩（慢，默认 `false`）
* **optimization\_level**：优化等级 (0-6，默认 2)

#### GIF

```bash
--gif-params quality=80
```

* **quality**：质量 (0-100，默认 80)

#### TIFF

```bash
--tiff-params algorithm=deflate,deflate_level=balanced
```

* **algorithm**：压缩算法（`uncompressed` / `lzw` / `deflate` / `packbits`，默认 `deflate`）
* **deflate\_level**：`fast` / `balanced` / `best`（默认 `balanced`）

#### WebP

```bash
--webp-params quality=80
```

* **quality**：质量 (0-100，默认 80)

---

### 尺寸调整

使用 `--resize-args` 参数指定规则和参数：

#### 格式

```
<rule>:key=value[,key=value...]
```

#### 可用规则

| 规则           | 说明        |
| ------------ | --------- |
| `no_resize`  | 不调整尺寸（默认） |
| `size`       | 指定宽高      |
| `scale`      | 按比例缩放     |
| `short_edge` | 设置短边长度    |
| `long_edge`  | 设置长边长度    |
| `width`      | 设置宽度      |
| `height`     | 设置高度      |

#### 可用键

* `edge_size=<pixels>`：短边或长边长度（依规则而定）
* `ratio=<0-1>`：宽高按比例缩放
* `w=<px|0-1>` / `h=<px|0-1>`：指定宽度或高度（像素或比例）
* `donot_enlarge=<true|false>`：是否禁止放大（默认 `false`）
* `keep_aspect_ratio=<bool>`：保持宽高比（默认 `true`）

#### 示例

```bash
--resize-args short_edge:edge_size=300
--resize-args size:w=800,h=600
--resize-args scale:ratio=0.8
--resize-args scale:w=0.8,h=0.7
```

---

## 示例

### 1. 压缩 JPEG 图片到 80% 质量

```bash
imgtool -i photo.jpg -o photo_comporess.jpg --jpeg-params quality=80
```

### 2. 批量压缩目录内 PNG 并转换为 WebP

```bash
imgtool -i ./images -o ./images-processed -t webp --webp-params quality=85
```

### 3. 按短边 300px 缩放并保留元数据

```bash
imgtool -i ./photos -o ./images-processed --resize-args short_edge:edge_size=300 --keep-metadata
```

### 4. 无损压缩并删除源文件

```bash
imgtool -i image.png -o image_compressed.png --lossless --delete-origin
```

更多示例见[example/example.sh](example/example.sh)
