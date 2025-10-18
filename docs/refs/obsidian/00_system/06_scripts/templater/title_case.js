const lower_arr = [
  "a",
  "an",
  "and",
  "as",
  "at",
  "but",
  "by",
  "for",
  "for",
  "from",
  "in",
  "into",
  "near",
  "no",
  "nor",
  "of",
  "on",
  "onto",
  "or",
  "pip",
  "the",
  "to",
  "with",
];

const upper_arr = [
  "AI",
  "API",
  "AWS",
  "BBQ",
  "BTB",
  "CASB",
  "CBS",
  "CEO",
  "CFO",
  "CHRO",
  "COO",
  "CPO",
  "CRM",
  "CSR",
  "CSS",
  "CSV",
  "CX",
  "DMP",
  "DSP",
  "EDA",
  "EPUB",
  "ERP",
  "GCP",
  "HTML",
  "HUJI",
  "IBI",
  "ID",
  "ISO",
  "JS",
  "JSON",
  "MBA",
  "MD",
  "ML",
  "MPLS",
  "ONA",
  "PCB",
  "PCI",
  "PDF",
  "PKM",
  "PPC",
  "QA",
  "RSA",
  "RTB",
  "SAAS",
  "SASE",
  "SEM",
  "SEO",
  "SQL",
  "SRS",
  "SSH",
  "TV",
  "UI",
  "UX",
  "UI/UX",
  "YAML",
  "ZMK",
  "ZTE",
];

async function title_case(str) {
  // EXP: Split the initial string by spaces.
  const initial_title = str
    .split(" ")
    .map((w) =>
      // EXP: If a word is in lower_arr, change it to lowercase
      // EXP: If a word is in upper_arr, change it to uppercase
      // EXP: Otherwise, title case the word
      lower_arr.includes(w.replaceAll(/^,|[,:]$/g, "").toLowerCase())
        ? w.toLowerCase()
        : upper_arr.includes(w.replaceAll(/^,|[,:]$/g, "").toUpperCase())
        ? w.toUpperCase()
        : w[0].toUpperCase() + w.substring(1).toLowerCase()
    )
    .join(" ");

  // EXP: Check the initial title for a colon
  const colon_index = initial_title.indexOf(":");

  let full_title;
  let title;
  let subtitle;
  if (colon_index === -1) {
    // EXP: If no colon is found,
    // EXP: assign the full title to the split initial title
    full_title = initial_title.split(" ");

    // EXP: Ensure first word is capitalized
    full_title[0] = lower_arr.includes(full_title[0])
      ? full_title[0].charAt(0).toUpperCase() + full_title[0].substring(1)
      : full_title[0];

    // EXP: Reassemble the full title as a string
    full_title = full_title.join(" ");
  } else {
    // EXP: If a colon is found,
    // EXP: split the initial title at the colon
    // EXP: into main and secondary titles
    // EXP: and follow the same procedure as above
    title = initial_title.split(":")[0].trim();
    title_arr = title.split(" ");
    title_arr[0] = lower_arr.includes(title_arr[0])
      ? title_arr[0].charAt(0).toUpperCase() + title_arr[0].substring(1)
      : title_arr[0];
    title = title_arr.join(" ");

    subtitle = initial_title.split(":")[1].trim();
    subtitle_arr = subtitle.split(" ");
    subtitle_arr[0] = lower_arr.includes(subtitle_arr[0])
      ? subtitle_arr[0].charAt(0).toUpperCase() + subtitle_arr[0].substring(1)
      : subtitle_arr[0];
    subtitle = subtitle_arr.join(" ");

    // EXP: Assign the full title to the rejoined title and subtitle
    full_title = `${title}: ${subtitle}`;
  }
  return full_title;
}

module.exports = title_case;
