---
title: city suggester
aliases:
  - City Suggester
  - city_suggester
  - city
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-23T11:18
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# City Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a city from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
let city_obj_arr = [
  { key: `Afula`, value: `afula` },
  { key: `Akko`, value: `akko` },
  { key: `Arad`, value: `arad` },
  { key: `Ashdod`, value: `ashdod` },
  { key: `Ashqelon`, value: `ashqelon` },
  { key: `Bat Yam`, value: `bat_yam` },
  { key: `Beer Sheba`, value: `beer_sheba` },
  { key: `Bet Shean`, value: `bet_shean` },
  { key: `Bet Shearim`, value: `bet_shearim` },
  { key: `Bnei Brak`, value: `bnei_brak` },
  { key: `Caesarea`, value: `caesarea` },
  { key: `Dimona`, value: `dimona` },
  { key: `Dor`, value: `dor` },
  { key: `Elat`, value: `elat` },
  { key: `En Gedi`, value: `en_gedi` },
  { key: `Givaatayim`, value: `givaatayim` },
  { key: `Hadera`, value: `hadera` },
  { key: `Haifa`, value: `haifa` },
  { key: `Herzliya`, value: `herzliya` },
  { key: `Hod HaSharon`, value: `hod_hasharon` },
  { key: `Holon`, value: `holon` },
  { key: `Jerusalem`, value: `jerusalem` },
  { key: `Karmiel`, value: `karmiel` },
  { key: `Kefar Sava`, value: `kefar_sava` },
  { key: `Lod`, value: `lod` },
  { key: `Meron`, value: `meron` },
  { key: `Nahariyya`, value: `nahariyya` },
  { key: `Nazareth`, value: `nazareth` },
  { key: `Netanya`, value: `netanya` },
  { key: `Petah Tiqwah`, value: `petah_tiqwah` },
  { key: `Qiryat Shemona`, value: `qiryat_shemona` },
  { key: `Raanana`, value: `raanana` },
  { key: `Ramat Gan`, value: `ramat_gan` },
  { key: `Ramla`, value: `ramla` },
  { key: `Rehovot`, value: `rehovot` },
  { key: `Rishon Leziyyon`, value: `rishon_leziyyon` },
  { key: `Rosh HaAyin`, value: `rosh_haayin` },
  { key: `Sedom`, value: `sedom` },
  { key: `Tel Aviv-Yafo`, value: `tel_aviv-yafo` },
  { key: `Tiberias`, value: `tiberias` },
  { key: `Yavne`, value: `yavne` },
  { key: `Zefat`, value: `zefat` },
  { key: `Abilene`, value: `abilene` },
  { key: `Akron`, value: `akron` },
  { key: `Albuquerque`, value: `albuquerque` },
  { key: `Alexandria`, value: `alexandria` },
  { key: `Allen`, value: `allen` },
  { key: `Allentown`, value: `allentown` },
  { key: `Amarillo`, value: `amarillo` },
  { key: `Anaheim`, value: `anaheim` },
  { key: `Anchorage`, value: `anchorage` },
  { key: `Ann Arbor`, value: `ann_arbor` },
  { key: `Antioch`, value: `antioch` },
  { key: `Arlington`, value: `arlington` },
  { key: `Arvada`, value: `arvada` },
  { key: `Athens`, value: `athens` },
  { key: `Atlanta`, value: `atlanta` },
  { key: `Augusta`, value: `augusta` },
  { key: `Aurora`, value: `aurora` },
  { key: `Austin`, value: `austin` },
  { key: `Bakersfield`, value: `bakersfield` },
  { key: `Baltimore`, value: `baltimore` },
  { key: `Baton Rouge`, value: `baton_rouge` },
  { key: `Beaumont`, value: `beaumont` },
  { key: `Bellevue`, value: `bellevue` },
  { key: `Bend`, value: `bend` },
  { key: `Berkeley`, value: `berkeley` },
  { key: `Billings`, value: `billings` },
  { key: `Birmingham`, value: `birmingham` },
  { key: `Boise`, value: `boise` },
  { key: `Boston`, value: `boston` },
  { key: `Boulder`, value: `boulder` },
  { key: `Bridgeport`, value: `bridgeport` },
  { key: `Brockton`, value: `brockton` },
  { key: `Broken Arrow`, value: `broken_arrow` },
  { key: `Brownsville`, value: `brownsville` },
  { key: `Buckeye`, value: `buckeye` },
  { key: `Buffalo`, value: `buffalo` },
  { key: `Burbank`, value: `burbank` },
  { key: `Cambridge`, value: `cambridge` },
  { key: `Cape Coral`, value: `cape_coral` },
  { key: `Carlsbad`, value: `carlsbad` },
  { key: `Carmel`, value: `carmel` },
  { key: `Carrollton`, value: `carrollton` },
  { key: `Cary`, value: `cary` },
  { key: `Cedar Rapids`, value: `cedar_rapids` },
  { key: `Centennial`, value: `centennial` },
  { key: `Chandler`, value: `chandler` },
  { key: `Charleston`, value: `charleston` },
  { key: `Charlotte`, value: `charlotte` },
  { key: `Chattanooga`, value: `chattanooga` },
  { key: `Chesapeake`, value: `chesapeake` },
  { key: `Chicago`, value: `chicago` },
  { key: `Chico`, value: `chico` },
  { key: `Chula Vista`, value: `chula_vista` },
  { key: `Cincinnati`, value: `cincinnati` },
  { key: `Clarksville`, value: `clarksville` },
  { key: `Clearwater`, value: `clearwater` },
  { key: `Cleveland`, value: `cleveland` },
  { key: `Clovis`, value: `clovis` },
  { key: `College Station`, value: `college_station` },
  { key: `Colorado Springs`, value: `colorado_springs` },
  { key: `Columbia`, value: `columbia` },
  { key: `Columbus`, value: `columbus` },
  { key: `Columbus`, value: `columbus` },
  { key: `Concord`, value: `concord` },
  { key: `Coral Springs`, value: `coral_springs` },
  { key: `Corona`, value: `corona` },
  { key: `Corpus Christi`, value: `corpus_christi` },
  { key: `Costa Mesa`, value: `costa_mesa` },
  { key: `Dallas`, value: `dallas` },
  { key: `Daly City`, value: `daly_city` },
  { key: `Davenport`, value: `davenport` },
  { key: `Davie`, value: `davie` },
  { key: `Dayton`, value: `dayton` },
  { key: `Dearborn`, value: `dearborn` },
  { key: `Denton`, value: `denton` },
  { key: `Denver`, value: `denver` },
  { key: `Des Moines`, value: `des_moines` },
  { key: `Detroit`, value: `detroit` },
  { key: `Downey`, value: `downey` },
  { key: `Durham`, value: `durham` },
  { key: `Edinburg`, value: `edinburg` },
  { key: `Edison`, value: `edison` },
  { key: `El Cajon`, value: `el_cajon` },
  { key: `El Monte`, value: `el_monte` },
  { key: `El Paso`, value: `el_paso` },
  { key: `Elgin`, value: `elgin` },
  { key: `Elizabeth`, value: `elizabeth` },
  { key: `Elk Grove`, value: `elk_grove` },
  { key: `Escondido`, value: `escondido` },
  { key: `Eugene`, value: `eugene` },
  { key: `Evansville`, value: `evansville` },
  { key: `Everett`, value: `everett` },
  { key: `Fairfield`, value: `fairfield` },
  { key: `Fargo`, value: `fargo` },
  { key: `Fayetteville`, value: `fayetteville` },
  { key: `Fishers`, value: `fishers` },
  { key: `Fontana`, value: `fontana` },
  { key: `Fort Collins`, value: `fort_collins` },
  { key: `Fort Lauderdale`, value: `fort_lauderdale` },
  { key: `Fort Wayne`, value: `fort_wayne` },
  { key: `Fort Worth`, value: `fort_worth` },
  { key: `Fremont`, value: `fremont` },
  { key: `Fresno`, value: `fresno` },
  { key: `Frisco`, value: `frisco` },
  { key: `Fullerton`, value: `fullerton` },
  { key: `Gainesville`, value: `gainesville` },
  { key: `Garden Grove`, value: `garden_grove` },
  { key: `Garland`, value: `garland` },
  { key: `Gilbert`, value: `gilbert` },
  { key: `Glendale`, value: `glendale` },
  { key: `Goodyear`, value: `goodyear` },
  { key: `Grand Prairie`, value: `grand_prairie` },
  { key: `Grand Rapids`, value: `grand_rapids` },
  { key: `Greeley`, value: `greeley` },
  { key: `Green Bay`, value: `green_bay` },
  { key: `Greensboro`, value: `greensboro` },
  { key: `Gresham`, value: `gresham` },
  { key: `Hampton`, value: `hampton` },
  { key: `Hartford`, value: `hartford` },
  { key: `Hayward`, value: `hayward` },
  { key: `Henderson`, value: `henderson` },
  { key: `Hesperia`, value: `hesperia` },
  { key: `Hialeah`, value: `hialeah` },
  { key: `High Point`, value: `high_point` },
  { key: `Hillsboro`, value: `hillsboro` },
  { key: `Hollywood`, value: `hollywood` },
  { key: `Honolulu`, value: `honolulu` },
  { key: `Houston`, value: `houston` },
  { key: `Huntington Beach`, value: `huntington_beach` },
  { key: `Huntsville`, value: `huntsville` },
  { key: `Independence`, value: `independence` },
  { key: `Indianapolis`, value: `indianapolis` },
  { key: `Inglewood`, value: `inglewood` },
  { key: `Irvine`, value: `irvine` },
  { key: `Irving`, value: `irving` },
  { key: `Jackson`, value: `jackson` },
  { key: `Jacksonville`, value: `jacksonville` },
  { key: `Jersey City`, value: `jersey_city` },
  { key: `Joliet`, value: `joliet` },
  { key: `Jurupa Valley`, value: `jurupa_valley` },
  { key: `Kansas City`, value: `kansas_city` },
  { key: `Kansas City`, value: `kansas_city` },
  { key: `Kent`, value: `kent` },
  { key: `Killeen`, value: `killeen` },
  { key: `Knoxville`, value: `knoxville` },
  { key: `Lafayette`, value: `lafayette` },
  { key: `Lakeland`, value: `lakeland` },
  { key: `Lakewood`, value: `lakewood` },
  { key: `Lancaster`, value: `lancaster` },
  { key: `Lansing`, value: `lansing` },
  { key: `Laredo`, value: `laredo` },
  { key: `Las Cruces`, value: `las_cruces` },
  { key: `Las Vegas`, value: `las_vegas` },
  { key: `League City`, value: `league_city` },
  { key: `Lee's Summit`, value: `lee's_summit` },
  { key: `Lewisville`, value: `lewisville` },
  { key: `Lexington`, value: `lexington` },
  { key: `Lincoln`, value: `lincoln` },
  { key: `Little Rock`, value: `little_rock` },
  { key: `Long Beach`, value: `long_beach` },
  { key: `Longmont`, value: `longmont` },
  { key: `Los Angeles`, value: `los_angeles` },
  { key: `Los Altos`, value: `los_altos` },
  { key: `Louisville`, value: `louisville` },
  { key: `Lowell`, value: `lowell` },
  { key: `Lubbock`, value: `lubbock` },
  { key: `Lynn`, value: `lynn` },
  { key: `Macon`, value: `macon` },
  { key: `Madison`, value: `madison` },
  { key: `Manchester`, value: `manchester` },
  { key: `McAllen`, value: `mcallen` },
  { key: `McKinney`, value: `mckinney` },
  { key: `Memphis`, value: `memphis` },
  { key: `Menifee`, value: `menifee` },
  { key: `Meridian`, value: `meridian` },
  { key: `Mesa`, value: `mesa` },
  { key: `Mesquite`, value: `mesquite` },
  { key: `Miami`, value: `miami` },
  { key: `Miami Gardens`, value: `miami_gardens` },
  { key: `Midland`, value: `midland` },
  { key: `Milwaukee`, value: `milwaukee` },
  { key: `Minneapolis`, value: `minneapolis` },
  { key: `Miramar`, value: `miramar` },
  { key: `Mobile`, value: `mobile` },
  { key: `Modesto`, value: `modesto` },
  { key: `Montgomery`, value: `montgomery` },
  { key: `Moreno Valley`, value: `moreno_valley` },
  { key: `Murfreesboro`, value: `murfreesboro` },
  { key: `Murrieta`, value: `murrieta` },
  { key: `Nampa`, value: `nampa` },
  { key: `Naperville`, value: `naperville` },
  { key: `Nashville`, value: `nashville` },
  { key: `New Bedford`, value: `new_bedford` },
  { key: `New Haven`, value: `new_haven` },
  { key: `New Orleans`, value: `new_orleans` },
  { key: `New York`, value: `new_york` },
  { key: `Newark`, value: `newark` },
  { key: `Newport News`, value: `newport_news` },
  { key: `Norfolk`, value: `norfolk` },
  { key: `Norman`, value: `norman` },
  { key: `North Charleston`, value: `north_charleston` },
  { key: `North Las Vegas`, value: `north_las_vegas` },
  { key: `Norwalk`, value: `norwalk` },
  { key: `Oakland`, value: `oakland` },
  { key: `Oceanside`, value: `oceanside` },
  { key: `Odessa`, value: `odessa` },
  { key: `Oklahoma City`, value: `oklahoma_city` },
  { key: `Olathe`, value: `olathe` },
  { key: `Omaha`, value: `omaha` },
  { key: `Ontario`, value: `ontario` },
  { key: `Orange`, value: `orange` },
  { key: `Orlando`, value: `orlando` },
  { key: `Overland Park`, value: `overland_park` },
  { key: `Oxnard`, value: `oxnard` },
  { key: `Palm Bay`, value: `palm_bay` },
  { key: `Palmdale`, value: `palmdale` },
  { key: `Pasadena`, value: `pasadena` },
  { key: `Paterson`, value: `paterson` },
  { key: `Pearland`, value: `pearland` },
  { key: `Pembroke Pines`, value: `pembroke_pines` },
  { key: `Peoria`, value: `peoria` },
  { key: `Philadelphia`, value: `philadelphia` },
  { key: `Phoenix`, value: `phoenix` },
  { key: `Pittsburgh`, value: `pittsburgh` },
  { key: `Plano`, value: `plano` },
  { key: `Pomona`, value: `pomona` },
  { key: `Pompano Beach`, value: `pompano_beach` },
  { key: `Port St. Lucie`, value: `port_st._lucie` },
  { key: `Portland`, value: `portland` },
  { key: `Providence`, value: `providence` },
  { key: `Provo`, value: `provo` },
  { key: `Pueblo`, value: `pueblo` },
  { key: `Quincy`, value: `quincy` },
  { key: `Raleigh`, value: `raleigh` },
  { key: `Rancho Cucamonga`, value: `rancho_cucamonga` },
  { key: `Reno`, value: `reno` },
  { key: `Renton`, value: `renton` },
  { key: `Rialto`, value: `rialto` },
  { key: `Richardson`, value: `richardson` },
  { key: `Richmond`, value: `richmond` },
  { key: `Richmond`, value: `richmond` },
  { key: `Rio Rancho`, value: `rio_rancho` },
  { key: `Riverside`, value: `riverside` },
  { key: `Rochester`, value: `rochester` },
  { key: `Rockford`, value: `rockford` },
  { key: `Roseville`, value: `roseville` },
  { key: `Round Rock`, value: `round_rock` },
  { key: `Sacramento`, value: `sacramento` },
  { key: `Saint Paul`, value: `saint_paul` },
  { key: `Salem`, value: `salem` },
  { key: `Salinas`, value: `salinas` },
  { key: `Salt Lake City`, value: `salt_lake_city` },
  { key: `San Antonio`, value: `san_antonio` },
  { key: `San Bernardino`, value: `san_bernardino` },
  { key: `San Diego`, value: `san_diego` },
  { key: `San Francisco`, value: `san_francisco` },
  { key: `San Jose`, value: `san_jose` },
  { key: `San Mateo`, value: `san_mateo` },
  { key: `Sandy Springs`, value: `sandy_springs` },
  { key: `Santa Ana`, value: `santa_ana` },
  { key: `Santa Clara`, value: `santa_clara` },
  { key: `Santa Clarita`, value: `santa_clarita` },
  { key: `Santa Maria`, value: `santa_maria` },
  { key: `Santa Rosa`, value: `santa_rosa` },
  { key: `Savannah`, value: `savannah` },
  { key: `Scottsdale`, value: `scottsdale` },
  { key: `Seattle`, value: `seattle` },
  { key: `Sebastopol`, value: `sebastopol` },
  { key: `Shreveport`, value: `shreveport` },
  { key: `Simi Valley`, value: `simi_valley` },
  { key: `Sioux Falls`, value: `sioux_falls` },
  { key: `South Bend`, value: `south_bend` },
  { key: `South Fulton`, value: `south_fulton` },
  { key: `Sparks`, value: `sparks` },
  { key: `Spokane`, value: `spokane` },
  { key: `Spokane Valley`, value: `spokane_valley` },
  { key: `Springfield`, value: `springfield` },
  { key: `St. Louis`, value: `st._louis` },
  { key: `St. Petersburg`, value: `st._petersburg` },
  { key: `Stamford`, value: `stamford` },
  { key: `Sterling Heights`, value: `sterling_heights` },
  { key: `Stockton`, value: `stockton` },
  { key: `Sugar Land`, value: `sugar_land` },
  { key: `Sunnyvale`, value: `sunnyvale` },
  { key: `Surprise`, value: `surprise` },
  { key: `Syracuse`, value: `syracuse` },
  { key: `Tacoma`, value: `tacoma` },
  { key: `Tallahassee`, value: `tallahassee` },
  { key: `Tampa`, value: `tampa` },
  { key: `Temecula`, value: `temecula` },
  { key: `Tempe`, value: `tempe` },
  { key: `Thornton`, value: `thornton` },
  { key: `Thousand Oaks`, value: `thousand_oaks` },
  { key: `Toledo`, value: `toledo` },
  { key: `Topeka`, value: `topeka` },
  { key: `Torrance`, value: `torrance` },
  { key: `Tucson`, value: `tucson` },
  { key: `Tulsa`, value: `tulsa` },
  { key: `Tuscaloosa`, value: `tuscaloosa` },
  { key: `Tyler`, value: `tyler` },
  { key: `Vacaville`, value: `vacaville` },
  { key: `Vallejo`, value: `vallejo` },
  { key: `Vancouver`, value: `vancouver` },
  { key: `Ventura`, value: `ventura` },
  { key: `Victorville`, value: `victorville` },
  { key: `Virginia Beach`, value: `virginia_beach` },
  { key: `Visalia`, value: `visalia` },
  { key: `Waco`, value: `waco` },
  { key: `Warren`, value: `warren` },
  { key: `Washington`, value: `washington` },
  { key: `Waterbury`, value: `waterbury` },
  { key: `West Covina`, value: `west_covina` },
  { key: `West Jordan`, value: `west_jordan` },
  { key: `West Palm Beach`, value: `west_palm_beach` },
  { key: `West Valley City`, value: `west_valley_city` },
  { key: `Westminster`, value: `westminster` },
  { key: `Wichita`, value: `wichita` },
  { key: `Wichita Falls`, value: `wichita_falls` },
  { key: `Wilmington`, value: `wilmington` },
  { key: `Winston-Salem`, value: `winston-salem` },
  { key: `Woodbridge`, value: `woodbridge` },
  { key: `Worcester`, value: `worcester` },
  { key: `Yonkers`, value: `yonkers` },
  { key: `Lisbon`, value: `lisbon` },
];

const obj_arr = [
  { key: `Null`, value: `null` },
  { key: `User Input`, value: `_user_input` },
];

obj_arr.push(city_obj_arr);

city_obj_arr = obj_arr.flat();

async function city(tp) {
  const city_obj = await tp.system.suggester(
    (item) => item.key,
    city_obj_arr,
    false,
    "City?"
  );

  let city_value = city_obj.value;

  if (city_value == "_user_input") {
    const city_input = await tp.system.prompt("City?");
    city_value = city_input.replaceAll(/\s/g, "_").toLowerCase();
  }

  return city_value;
}

module.exports = city;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CITY
//---------------------------------------------------------
const city = await tp.user.city(tp);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[71_00_book|Book Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[city.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[country|Country Suggester]]
2. [[Country and City Suggester]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
