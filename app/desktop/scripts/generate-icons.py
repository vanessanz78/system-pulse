from pathlib import Path
import math
import struct
import zlib


ROOT = Path(__file__).resolve().parents[1] / "src-tauri" / "icons"


def heart_value(x: float, y: float) -> float:
    return (x * x + y * y - 1) ** 3 - x * x * y**3


def write_png(path: Path, size: int) -> bytes:
    scale = 4
    big = size * scale
    center = big / 2
    radius = big * 0.43
    rows: list[bytes] = []

    for y in range(size):
        row: list[int] = []
        for x in range(size):
            accum = [0, 0, 0, 0]
            for sy in range(scale):
                for sx in range(scale):
                    px = x * scale + sx + 0.5
                    py = y * scale + sy + 0.5
                    dx = px - center
                    dy = py - center
                    dist = math.hypot(dx, dy)

                    if dist > radius * 1.10:
                        rgba = (0, 0, 0, 0)
                    else:
                        background = (7, 16, 15, 255)
                        halo = max(0.0, 1.0 - abs(dist - radius * 0.73) / (radius * 0.12))
                        if halo > 0:
                            rgba = (
                                int(background[0] * (1 - halo) + 120 * halo),
                                int(background[1] * (1 - halo) + 212 * halo),
                                int(background[2] * (1 - halo) + 90 * halo),
                                255,
                            )
                        else:
                            rgba = background

                        hx = dx / (radius * 0.52)
                        hy = -dy / (radius * 0.52) + 0.10
                        if heart_value(hx, hy) <= 0:
                            rgba = (255, 255, 255, 255)

                    for channel in range(4):
                        accum[channel] += rgba[channel]
            row.extend(value // (scale * scale) for value in accum)
        rows.append(bytes(row))

    raw = b"".join(b"\x00" + row for row in rows)

    def chunk(kind: bytes, data: bytes) -> bytes:
        checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    path.write_bytes(png)
    return png


def write_ico(path: Path, icon_32: bytes, icon_256: bytes) -> None:
    entries = [(32, icon_32), (256, icon_256)]
    header = struct.pack("<HHH", 0, 1, len(entries))
    offset = 6 + 16 * len(entries)
    directory = b""
    payloads = []

    for size, payload in entries:
        dimension = 0 if size == 256 else size
        directory += struct.pack(
            "<BBBBHHII",
            dimension,
            dimension,
            0,
            0,
            1,
            32,
            len(payload),
            offset,
        )
        payloads.append(payload)
        offset += len(payload)

    path.write_bytes(header + directory + b"".join(payloads))


def write_icns(path: Path, icon_128: bytes, icon_256: bytes) -> None:
    chunks = [(b"ic07", icon_128), (b"ic08", icon_256)]
    body = b"".join(kind + struct.pack(">I", len(data) + 8) + data for kind, data in chunks)
    path.write_bytes(b"icns" + struct.pack(">I", len(body) + 8) + body)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    icon_32 = write_png(ROOT / "32x32.png", 32)
    icon_128 = write_png(ROOT / "128x128.png", 128)
    icon_256 = write_png(ROOT / "128x128@2x.png", 256)
    write_ico(ROOT / "icon.ico", icon_32, icon_256)
    write_icns(ROOT / "icon.icns", icon_128, icon_256)
    print(f"Generated Tauri icons in {ROOT}")


if __name__ == "__main__":
    main()
