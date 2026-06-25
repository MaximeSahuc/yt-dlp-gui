/** Subtitle parsing and bilingual merge utilities */

interface SubBlock {
  timing: string;
  text: string;
}

/** Parse an SRT subtitle file into a list of blocks */
export const parseSrtBlocks = (content: string): SubBlock[] => {
  const blocks: SubBlock[] = [];
  const normalized = content.replace(/\r/g, "");
  const parts = normalized.split("\n\n");

  for (const part of parts) {
    const lines = part.trim().split("\n");
    if (lines.length >= 3) {
      // line 1 is the index, line 2 is the timecode, rest is text
      blocks.push({ timing: lines[1], text: lines.slice(2).join("\n") });
    }
  }

  return blocks;
};

/** Parse a VTT subtitle file into a list of blocks */
export const parseVttBlocks = (content: string): SubBlock[] => {
  const blocks: SubBlock[] = [];
  const normalized = content.replace(/\r/g, "");
  const parts = normalized.split("\n\n");

  for (const part of parts) {
    const lines = part.trim().split("\n");
    if (!lines.length) continue;

    // find the line containing -->
    const timingIdx = lines.findIndex((l) => l.includes("-->"));
    if (timingIdx >= 0 && timingIdx + 1 < lines.length) {
      blocks.push({
        timing: lines[timingIdx],
        text: lines.slice(timingIdx + 1).join("\n"),
      });
    }
  }

  return blocks;
};

/** Merge two SRT subtitle tracks into a bilingual subtitle */
export const mergeBilingualSrt = (primary: string, secondary: string): string => {
  const pBlocks = parseSrtBlocks(primary);
  const sBlocks = parseSrtBlocks(secondary);
  const maxLen = Math.max(pBlocks.length, sBlocks.length);

  const lines: string[] = [];
  for (let i = 0; i < maxLen; i++) {
    lines.push(String(i + 1));
    const pb = pBlocks[i];
    const sb = sBlocks[i];

    if (pb) {
      lines.push(pb.timing);
      lines.push(pb.text);
      if (sb) lines.push(sb.text);
    } else if (sb) {
      lines.push(sb.timing);
      lines.push(sb.text);
    }
    lines.push(""); // blank line separator
  }

  return lines.join("\r\n");
};

/** Merge two VTT subtitle tracks into a bilingual subtitle */
export const mergeBilingualVtt = (primary: string, secondary: string): string => {
  const pBlocks = parseVttBlocks(primary);
  const sBlocks = parseVttBlocks(secondary);
  const maxLen = Math.max(pBlocks.length, sBlocks.length);

  const lines: string[] = ["WEBVTT", ""];
  for (let i = 0; i < maxLen; i++) {
    const pb = pBlocks[i];
    const sb = sBlocks[i];

    if (pb) {
      lines.push(pb.timing);
      lines.push(pb.text);
      if (sb) lines.push(sb.text);
    } else if (sb) {
      lines.push(sb.timing);
      lines.push(sb.text);
    }
    lines.push(""); // blank line separator
  }

  return lines.join("\r\n");
};
