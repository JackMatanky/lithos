<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const org_name_alias = "52_organization_name_alias";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;

const org_specialties = await tp.system.prompt(
  "Organization's specialties?",
  null,
  false,
  false
);
let specialties_name = "null";
let specialties_value = "null";
let specialties_tag;

if (org_specialties.match(/\w/g)) {
  specialties_name = (await tp.user.title_case(org_specialties))
    .replaceAll(/&/g, "and")
    .replace(/(\s|,\s)(and)\s([\w\s]+)$/g, ", $3");
  specialties_value = specialties_name
    .split(", ")
    .map((s) => yaml_li(s))
    .join("");
  specialties_tag = specialties_name
    .split(", ")
    .map((s) =>
      s.trim().replaceAll(`"`, "").replaceAll(/[\s-]/g, "_").toLowerCase()
    )
    .join(" ");
}

tR += org_specialties;
tR += new_line;
tR += new_line;
tR += specialties_name;
tR += new_line;
tR += new_line;
tR += specialties_value;
tR += new_line;
tR += new_line;
tR += specialties_tag;
%>


specialties: <%* tR += specialties_value %>
