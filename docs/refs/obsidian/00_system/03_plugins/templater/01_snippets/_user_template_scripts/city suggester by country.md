---
title: city suggester by country
aliases:
  - City Suggester Filtered by Country
  - city_country_suggester
  - city_country
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-23T11:18
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# City Suggester Filtered by Country

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Choose a city from a suggester based on previously defined country value.

---

## Snippet

```javascript
const city_obj_arr = [
  { country: `null`, key: `Null`, value: `null` },
  { country: `israel`, key: `Afula`, value: `afula` },
  { country: `israel`, key: `Akko`, value: `akko` },
  { country: `israel`, key: `Arad`, value: `arad` },
  { country: `israel`, key: `Ashdod`, value: `ashdod` },
  { country: `israel`, key: `Ashqelon`, value: `ashqelon` },
  { country: `israel`, key: `Bat Yam`, value: `bat_yam` },
  { country: `israel`, key: `Beer Sheba`, value: `beer_sheba` },
  { country: `israel`, key: `Bet Shean`, value: `bet_shean` },
  { country: `israel`, key: `Bet Shearim`, value: `bet_shearim` },
  { country: `israel`, key: `Bnei Brak`, value: `bnei_brak` },
  { country: `israel`, key: `Caesarea`, value: `caesarea` },
  { country: `israel`, key: `Dimona`, value: `dimona` },
  { country: `israel`, key: `Dor`, value: `dor` },
  { country: `israel`, key: `Elat`, value: `elat` },
  { country: `israel`, key: `En Gedi`, value: `en_gedi` },
  { country: `israel`, key: `Givaatayim`, value: `givaatayim` },
  { country: `israel`, key: `Hadera`, value: `hadera` },
  { country: `israel`, key: `Haifa`, value: `haifa` },
  { country: `israel`, key: `Herzliya`, value: `herzliya` },
  { country: `israel`, key: `Hod HaSharon`, value: `hod_hasharon` },
  { country: `israel`, key: `Holon`, value: `holon` },
  { country: `israel`, key: `Jerusalem`, value: `jerusalem` },
  { country: `israel`, key: `Karmiel`, value: `karmiel` },
  { country: `israel`, key: `Kefar Sava`, value: `kefar_sava` },
  { country: `israel`, key: `Lod`, value: `lod` },
  { country: `israel`, key: `Meron`, value: `meron` },
  { country: `israel`, key: `Nahariyya`, value: `nahariyya` },
  { country: `israel`, key: `Nazareth`, value: `nazareth` },
  { country: `israel`, key: `Netanya`, value: `netanya` },
  { country: `israel`, key: `Petah Tiqwah`, value: `petah_tiqwah` },
  { country: `israel`, key: `Qiryat Shemona`, value: `qiryat_shemona` },
  { country: `israel`, key: `Raanana`, value: `raanana` },
  { country: `israel`, key: `Ramat Gan`, value: `ramat_gan` },
  { country: `israel`, key: `Ramla`, value: `ramla` },
  { country: `israel`, key: `Rehovot`, value: `rehovot` },
  { country: `israel`, key: `Rishon Leziyyon`, value: `rishon_leziyyon` },
  { country: `israel`, key: `Rosh HaAyin`, value: `rosh_haayin` },
  { country: `israel`, key: `Sedom`, value: `sedom` },
  { country: `israel`, key: `Tel Aviv-Yafo`, value: `tel_aviv-yafo` },
  { country: `israel`, key: `Tiberias`, value: `tiberias` },
  { country: `israel`, key: `Yavne`, value: `yavne` },
  { country: `israel`, key: `Zefat`, value: `zefat` },
  { country: `usa`, key: `Abilene`, value: `abilene` },
  { country: `usa`, key: `Akron`, value: `akron` },
  { country: `usa`, key: `Albuquerque`, value: `albuquerque` },
  { country: `usa`, key: `Alexandria`, value: `alexandria` },
  { country: `usa`, key: `Allen`, value: `allen` },
  { country: `usa`, key: `Allentown`, value: `allentown` },
  { country: `usa`, key: `Amarillo`, value: `amarillo` },
  { country: `usa`, key: `Anaheim`, value: `anaheim` },
  { country: `usa`, key: `Anchorage`, value: `anchorage` },
  { country: `usa`, key: `Ann Arbor`, value: `ann_arbor` },
  { country: `usa`, key: `Antioch`, value: `antioch` },
  { country: `usa`, key: `Arlington`, value: `arlington` },
  { country: `usa`, key: `Arvada`, value: `arvada` },
  { country: `usa`, key: `Athens`, value: `athens` },
  { country: `usa`, key: `Atlanta`, value: `atlanta` },
  { country: `usa`, key: `Augusta`, value: `augusta` },
  { country: `usa`, key: `Aurora`, value: `aurora` },
  { country: `usa`, key: `Austin`, value: `austin` },
  { country: `usa`, key: `Bakersfield`, value: `bakersfield` },
  { country: `usa`, key: `Baltimore`, value: `baltimore` },
  { country: `usa`, key: `Baton Rouge`, value: `baton_rouge` },
  { country: `usa`, key: `Beaumont`, value: `beaumont` },
  { country: `usa`, key: `Bellevue`, value: `bellevue` },
  { country: `usa`, key: `Bend`, value: `bend` },
  { country: `usa`, key: `Berkeley`, value: `berkeley` },
  { country: `usa`, key: `Billings`, value: `billings` },
  { country: `usa`, key: `Birmingham`, value: `birmingham` },
  { country: `usa`, key: `Boise`, value: `boise` },
  { country: `usa`, key: `Boston`, value: `boston` },
  { country: `usa`, key: `Boulder`, value: `boulder` },
  { country: `usa`, key: `Bridgeport`, value: `bridgeport` },
  { country: `usa`, key: `Brockton`, value: `brockton` },
  { country: `usa`, key: `Broken Arrow`, value: `broken_arrow` },
  { country: `usa`, key: `Brownsville`, value: `brownsville` },
  { country: `usa`, key: `Buckeye`, value: `buckeye` },
  { country: `usa`, key: `Buffalo`, value: `buffalo` },
  { country: `usa`, key: `Burbank`, value: `burbank` },
  { country: `usa`, key: `Cambridge`, value: `cambridge` },
  { country: `usa`, key: `Cape Coral`, value: `cape_coral` },
  { country: `usa`, key: `Carlsbad`, value: `carlsbad` },
  { country: `usa`, key: `Carmel`, value: `carmel` },
  { country: `usa`, key: `Carrollton`, value: `carrollton` },
  { country: `usa`, key: `Cary`, value: `cary` },
  { country: `usa`, key: `Cedar Rapids`, value: `cedar_rapids` },
  { country: `usa`, key: `Centennial`, value: `centennial` },
  { country: `usa`, key: `Chandler`, value: `chandler` },
  { country: `usa`, key: `Charleston`, value: `charleston` },
  { country: `usa`, key: `Charlotte`, value: `charlotte` },
  { country: `usa`, key: `Chattanooga`, value: `chattanooga` },
  { country: `usa`, key: `Chesapeake`, value: `chesapeake` },
  { country: `usa`, key: `Chicago`, value: `chicago` },
  { country: `usa`, key: `Chico`, value: `chico` },
  { country: `usa`, key: `Chula Vista`, value: `chula_vista` },
  { country: `usa`, key: `Cincinnati`, value: `cincinnati` },
  { country: `usa`, key: `Clarksville`, value: `clarksville` },
  { country: `usa`, key: `Clearwater`, value: `clearwater` },
  { country: `usa`, key: `Cleveland`, value: `cleveland` },
  { country: `usa`, key: `Clovis`, value: `clovis` },
  { country: `usa`, key: `College Station`, value: `college_station` },
  { country: `usa`, key: `Colorado Springs`, value: `colorado_springs` },
  { country: `usa`, key: `Columbia`, value: `columbia` },
  { country: `usa`, key: `Columbus`, value: `columbus` },
  { country: `usa`, key: `Columbus`, value: `columbus` },
  { country: `usa`, key: `Concord`, value: `concord` },
  { country: `usa`, key: `Coral Springs`, value: `coral_springs` },
  { country: `usa`, key: `Corona`, value: `corona` },
  { country: `usa`, key: `Corpus Christi`, value: `corpus_christi` },
  { country: `usa`, key: `Costa Mesa`, value: `costa_mesa` },
  { country: `usa`, key: `Dallas`, value: `dallas` },
  { country: `usa`, key: `Daly City`, value: `daly_city` },
  { country: `usa`, key: `Davenport`, value: `davenport` },
  { country: `usa`, key: `Davie`, value: `davie` },
  { country: `usa`, key: `Dayton`, value: `dayton` },
  { country: `usa`, key: `Dearborn`, value: `dearborn` },
  { country: `usa`, key: `Denton`, value: `denton` },
  { country: `usa`, key: `Denver`, value: `denver` },
  { country: `usa`, key: `Des Moines`, value: `des_moines` },
  { country: `usa`, key: `Detroit`, value: `detroit` },
  { country: `usa`, key: `Downey`, value: `downey` },
  { country: `usa`, key: `Durham`, value: `durham` },
  { country: `usa`, key: `Edinburg`, value: `edinburg` },
  { country: `usa`, key: `Edison`, value: `edison` },
  { country: `usa`, key: `El Cajon`, value: `el_cajon` },
  { country: `usa`, key: `El Monte`, value: `el_monte` },
  { country: `usa`, key: `El Paso`, value: `el_paso` },
  { country: `usa`, key: `Elgin`, value: `elgin` },
  { country: `usa`, key: `Elizabeth`, value: `elizabeth` },
  { country: `usa`, key: `Elk Grove`, value: `elk_grove` },
  { country: `usa`, key: `Escondido`, value: `escondido` },
  { country: `usa`, key: `Eugene`, value: `eugene` },
  { country: `usa`, key: `Evansville`, value: `evansville` },
  { country: `usa`, key: `Everett`, value: `everett` },
  { country: `usa`, key: `Fairfield`, value: `fairfield` },
  { country: `usa`, key: `Fargo`, value: `fargo` },
  { country: `usa`, key: `Fayetteville`, value: `fayetteville` },
  { country: `usa`, key: `Fishers`, value: `fishers` },
  { country: `usa`, key: `Fontana`, value: `fontana` },
  { country: `usa`, key: `Fort Collins`, value: `fort_collins` },
  { country: `usa`, key: `Fort Lauderdale`, value: `fort_lauderdale` },
  { country: `usa`, key: `Fort Wayne`, value: `fort_wayne` },
  { country: `usa`, key: `Fort Worth`, value: `fort_worth` },
  { country: `usa`, key: `Fremont`, value: `fremont` },
  { country: `usa`, key: `Fresno`, value: `fresno` },
  { country: `usa`, key: `Frisco`, value: `frisco` },
  { country: `usa`, key: `Fullerton`, value: `fullerton` },
  { country: `usa`, key: `Gainesville`, value: `gainesville` },
  { country: `usa`, key: `Garden Grove`, value: `garden_grove` },
  { country: `usa`, key: `Garland`, value: `garland` },
  { country: `usa`, key: `Gilbert`, value: `gilbert` },
  { country: `usa`, key: `Glendale`, value: `glendale` },
  { country: `usa`, key: `Goodyear`, value: `goodyear` },
  { country: `usa`, key: `Grand Prairie`, value: `grand_prairie` },
  { country: `usa`, key: `Grand Rapids`, value: `grand_rapids` },
  { country: `usa`, key: `Greeley`, value: `greeley` },
  { country: `usa`, key: `Green Bay`, value: `green_bay` },
  { country: `usa`, key: `Greensboro`, value: `greensboro` },
  { country: `usa`, key: `Gresham`, value: `gresham` },
  { country: `usa`, key: `Hampton`, value: `hampton` },
  { country: `usa`, key: `Hartford`, value: `hartford` },
  { country: `usa`, key: `Hayward`, value: `hayward` },
  { country: `usa`, key: `Henderson`, value: `henderson` },
  { country: `usa`, key: `Hesperia`, value: `hesperia` },
  { country: `usa`, key: `Hialeah`, value: `hialeah` },
  { country: `usa`, key: `High Point`, value: `high_point` },
  { country: `usa`, key: `Hillsboro`, value: `hillsboro` },
  { country: `usa`, key: `Hollywood`, value: `hollywood` },
  { country: `usa`, key: `Honolulu`, value: `honolulu` },
  { country: `usa`, key: `Houston`, value: `houston` },
  { country: `usa`, key: `Huntington Beach`, value: `huntington_beach` },
  { country: `usa`, key: `Huntsville`, value: `huntsville` },
  { country: `usa`, key: `Independence`, value: `independence` },
  { country: `usa`, key: `Indianapolis`, value: `indianapolis` },
  { country: `usa`, key: `Inglewood`, value: `inglewood` },
  { country: `usa`, key: `Irvine`, value: `irvine` },
  { country: `usa`, key: `Irving`, value: `irving` },
  { country: `usa`, key: `Jackson`, value: `jackson` },
  { country: `usa`, key: `Jacksonville`, value: `jacksonville` },
  { country: `usa`, key: `Jersey City`, value: `jersey_city` },
  { country: `usa`, key: `Joliet`, value: `joliet` },
  { country: `usa`, key: `Jurupa Valley`, value: `jurupa_valley` },
  { country: `usa`, key: `Kansas City`, value: `kansas_city` },
  { country: `usa`, key: `Kansas City`, value: `kansas_city` },
  { country: `usa`, key: `Kent`, value: `kent` },
  { country: `usa`, key: `Killeen`, value: `killeen` },
  { country: `usa`, key: `Knoxville`, value: `knoxville` },
  { country: `usa`, key: `Lafayette`, value: `lafayette` },
  { country: `usa`, key: `Lakeland`, value: `lakeland` },
  { country: `usa`, key: `Lakewood`, value: `lakewood` },
  { country: `usa`, key: `Lancaster`, value: `lancaster` },
  { country: `usa`, key: `Lansing`, value: `lansing` },
  { country: `usa`, key: `Laredo`, value: `laredo` },
  { country: `usa`, key: `Las Cruces`, value: `las_cruces` },
  { country: `usa`, key: `Las Vegas`, value: `las_vegas` },
  { country: `usa`, key: `League City`, value: `league_city` },
  { country: `usa`, key: `Lee's Summit`, value: `lee's_summit` },
  { country: `usa`, key: `Lewisville`, value: `lewisville` },
  { country: `usa`, key: `Lexington`, value: `lexington` },
  { country: `usa`, key: `Lincoln`, value: `lincoln` },
  { country: `usa`, key: `Little Rock`, value: `little_rock` },
  { country: `usa`, key: `Long Beach`, value: `long_beach` },
  { country: `usa`, key: `Longmont`, value: `longmont` },
  { country: `usa`, key: `Los Angeles`, value: `los_angeles` },
  { country: `usa`, key: `Louisville`, value: `louisville` },
  { country: `usa`, key: `Lowell`, value: `lowell` },
  { country: `usa`, key: `Lubbock`, value: `lubbock` },
  { country: `usa`, key: `Lynn`, value: `lynn` },
  { country: `usa`, key: `Macon`, value: `macon` },
  { country: `usa`, key: `Madison`, value: `madison` },
  { country: `usa`, key: `Manchester`, value: `manchester` },
  { country: `usa`, key: `McAllen`, value: `mcallen` },
  { country: `usa`, key: `McKinney`, value: `mckinney` },
  { country: `usa`, key: `Memphis`, value: `memphis` },
  { country: `usa`, key: `Menifee`, value: `menifee` },
  { country: `usa`, key: `Meridian`, value: `meridian` },
  { country: `usa`, key: `Mesa`, value: `mesa` },
  { country: `usa`, key: `Mesquite`, value: `mesquite` },
  { country: `usa`, key: `Miami`, value: `miami` },
  { country: `usa`, key: `Miami Gardens`, value: `miami_gardens` },
  { country: `usa`, key: `Midland`, value: `midland` },
  { country: `usa`, key: `Milwaukee`, value: `milwaukee` },
  { country: `usa`, key: `Minneapolis`, value: `minneapolis` },
  { country: `usa`, key: `Miramar`, value: `miramar` },
  { country: `usa`, key: `Mobile`, value: `mobile` },
  { country: `usa`, key: `Modesto`, value: `modesto` },
  { country: `usa`, key: `Montgomery`, value: `montgomery` },
  { country: `usa`, key: `Moreno Valley`, value: `moreno_valley` },
  { country: `usa`, key: `Murfreesboro`, value: `murfreesboro` },
  { country: `usa`, key: `Murrieta`, value: `murrieta` },
  { country: `usa`, key: `Nampa`, value: `nampa` },
  { country: `usa`, key: `Naperville`, value: `naperville` },
  { country: `usa`, key: `Nashville`, value: `nashville` },
  { country: `usa`, key: `New Bedford`, value: `new_bedford` },
  { country: `usa`, key: `New Haven`, value: `new_haven` },
  { country: `usa`, key: `New Orleans`, value: `new_orleans` },
  { country: `usa`, key: `New York`, value: `new_york` },
  { country: `usa`, key: `Newark`, value: `newark` },
  { country: `usa`, key: `Newport News`, value: `newport_news` },
  { country: `usa`, key: `Norfolk`, value: `norfolk` },
  { country: `usa`, key: `Norman`, value: `norman` },
  { country: `usa`, key: `North Charleston`, value: `north_charleston` },
  { country: `usa`, key: `North Las Vegas`, value: `north_las_vegas` },
  { country: `usa`, key: `Norwalk`, value: `norwalk` },
  { country: `usa`, key: `Oakland`, value: `oakland` },
  { country: `usa`, key: `Oceanside`, value: `oceanside` },
  { country: `usa`, key: `Odessa`, value: `odessa` },
  { country: `usa`, key: `Oklahoma City`, value: `oklahoma_city` },
  { country: `usa`, key: `Olathe`, value: `olathe` },
  { country: `usa`, key: `Omaha`, value: `omaha` },
  { country: `usa`, key: `Ontario`, value: `ontario` },
  { country: `usa`, key: `Orange`, value: `orange` },
  { country: `usa`, key: `Orlando`, value: `orlando` },
  { country: `usa`, key: `Overland Park`, value: `overland_park` },
  { country: `usa`, key: `Oxnard`, value: `oxnard` },
  { country: `usa`, key: `Palm Bay`, value: `palm_bay` },
  { country: `usa`, key: `Palmdale`, value: `palmdale` },
  { country: `usa`, key: `Pasadena`, value: `pasadena` },
  { country: `usa`, key: `Paterson`, value: `paterson` },
  { country: `usa`, key: `Pearland`, value: `pearland` },
  { country: `usa`, key: `Pembroke Pines`, value: `pembroke_pines` },
  { country: `usa`, key: `Peoria`, value: `peoria` },
  { country: `usa`, key: `Philadelphia`, value: `philadelphia` },
  { country: `usa`, key: `Phoenix`, value: `phoenix` },
  { country: `usa`, key: `Pittsburgh`, value: `pittsburgh` },
  { country: `usa`, key: `Plano`, value: `plano` },
  { country: `usa`, key: `Pomona`, value: `pomona` },
  { country: `usa`, key: `Pompano Beach`, value: `pompano_beach` },
  { country: `usa`, key: `Port St. Lucie`, value: `port_st._lucie` },
  { country: `usa`, key: `Portland`, value: `portland` },
  { country: `usa`, key: `Providence`, value: `providence` },
  { country: `usa`, key: `Provo`, value: `provo` },
  { country: `usa`, key: `Pueblo`, value: `pueblo` },
  { country: `usa`, key: `Quincy`, value: `quincy` },
  { country: `usa`, key: `Raleigh`, value: `raleigh` },
  { country: `usa`, key: `Rancho Cucamonga`, value: `rancho_cucamonga` },
  { country: `usa`, key: `Reno`, value: `reno` },
  { country: `usa`, key: `Renton`, value: `renton` },
  { country: `usa`, key: `Rialto`, value: `rialto` },
  { country: `usa`, key: `Richardson`, value: `richardson` },
  { country: `usa`, key: `Richmond`, value: `richmond` },
  { country: `usa`, key: `Richmond`, value: `richmond` },
  { country: `usa`, key: `Rio Rancho`, value: `rio_rancho` },
  { country: `usa`, key: `Riverside`, value: `riverside` },
  { country: `usa`, key: `Rochester`, value: `rochester` },
  { country: `usa`, key: `Rockford`, value: `rockford` },
  { country: `usa`, key: `Roseville`, value: `roseville` },
  { country: `usa`, key: `Round Rock`, value: `round_rock` },
  { country: `usa`, key: `Sacramento`, value: `sacramento` },
  { country: `usa`, key: `Saint Paul`, value: `saint_paul` },
  { country: `usa`, key: `Salem`, value: `salem` },
  { country: `usa`, key: `Salinas`, value: `salinas` },
  { country: `usa`, key: `Salt Lake City`, value: `salt_lake_city` },
  { country: `usa`, key: `San Antonio`, value: `san_antonio` },
  { country: `usa`, key: `San Bernardino`, value: `san_bernardino` },
  { country: `usa`, key: `San Diego`, value: `san_diego` },
  { country: `usa`, key: `San Francisco`, value: `san_francisco` },
  { country: `usa`, key: `San Jose`, value: `san_jose` },
  { country: `usa`, key: `San Mateo`, value: `san_mateo` },
  { country: `usa`, key: `Sandy Springs`, value: `sandy_springs` },
  { country: `usa`, key: `Santa Ana`, value: `santa_ana` },
  { country: `usa`, key: `Santa Clara`, value: `santa_clara` },
  { country: `usa`, key: `Santa Clarita`, value: `santa_clarita` },
  { country: `usa`, key: `Santa Maria`, value: `santa_maria` },
  { country: `usa`, key: `Santa Rosa`, value: `santa_rosa` },
  { country: `usa`, key: `Savannah`, value: `savannah` },
  { country: `usa`, key: `Scottsdale`, value: `scottsdale` },
  { country: `usa`, key: `Seattle`, value: `seattle` },
  { country: `usa`, key: `Shreveport`, value: `shreveport` },
  { country: `usa`, key: `Simi Valley`, value: `simi_valley` },
  { country: `usa`, key: `Sioux Falls`, value: `sioux_falls` },
  { country: `usa`, key: `South Bend`, value: `south_bend` },
  { country: `usa`, key: `South Fulton`, value: `south_fulton` },
  { country: `usa`, key: `Sparks`, value: `sparks` },
  { country: `usa`, key: `Spokane`, value: `spokane` },
  { country: `usa`, key: `Spokane Valley`, value: `spokane_valley` },
  { country: `usa`, key: `Springfield`, value: `springfield` },
  { country: `usa`, key: `St. Louis`, value: `st._louis` },
  { country: `usa`, key: `St. Petersburg`, value: `st._petersburg` },
  { country: `usa`, key: `Stamford`, value: `stamford` },
  { country: `usa`, key: `Sterling Heights`, value: `sterling_heights` },
  { country: `usa`, key: `Stockton`, value: `stockton` },
  { country: `usa`, key: `Sugar Land`, value: `sugar_land` },
  { country: `usa`, key: `Sunnyvale`, value: `sunnyvale` },
  { country: `usa`, key: `Surprise`, value: `surprise` },
  { country: `usa`, key: `Syracuse`, value: `syracuse` },
  { country: `usa`, key: `Tacoma`, value: `tacoma` },
  { country: `usa`, key: `Tallahassee`, value: `tallahassee` },
  { country: `usa`, key: `Tampa`, value: `tampa` },
  { country: `usa`, key: `Temecula`, value: `temecula` },
  { country: `usa`, key: `Tempe`, value: `tempe` },
  { country: `usa`, key: `Thornton`, value: `thornton` },
  { country: `usa`, key: `Thousand Oaks`, value: `thousand_oaks` },
  { country: `usa`, key: `Toledo`, value: `toledo` },
  { country: `usa`, key: `Topeka`, value: `topeka` },
  { country: `usa`, key: `Torrance`, value: `torrance` },
  { country: `usa`, key: `Tucson`, value: `tucson` },
  { country: `usa`, key: `Tulsa`, value: `tulsa` },
  { country: `usa`, key: `Tuscaloosa`, value: `tuscaloosa` },
  { country: `usa`, key: `Tyler`, value: `tyler` },
  { country: `usa`, key: `Vacaville`, value: `vacaville` },
  { country: `usa`, key: `Vallejo`, value: `vallejo` },
  { country: `usa`, key: `Vancouver`, value: `vancouver` },
  { country: `usa`, key: `Ventura`, value: `ventura` },
  { country: `usa`, key: `Victorville`, value: `victorville` },
  { country: `usa`, key: `Virginia Beach`, value: `virginia_beach` },
  { country: `usa`, key: `Visalia`, value: `visalia` },
  { country: `usa`, key: `Waco`, value: `waco` },
  { country: `usa`, key: `Warren`, value: `warren` },
  { country: `usa`, key: `Washington`, value: `washington` },
  { country: `usa`, key: `Waterbury`, value: `waterbury` },
  { country: `usa`, key: `West Covina`, value: `west_covina` },
  { country: `usa`, key: `West Jordan`, value: `west_jordan` },
  { country: `usa`, key: `West Palm Beach`, value: `west_palm_beach` },
  { country: `usa`, key: `West Valley City`, value: `west_valley_city` },
  { country: `usa`, key: `Westminster`, value: `westminster` },
  { country: `usa`, key: `Wichita`, value: `wichita` },
  { country: `usa`, key: `Wichita Falls`, value: `wichita_falls` },
  { country: `usa`, key: `Wilmington`, value: `wilmington` },
  { country: `usa`, key: `Winston-Salem`, value: `winston-salem` },
  { country: `usa`, key: `Woodbridge`, value: `woodbridge` },
  { country: `usa`, key: `Worcester`, value: `worcester` },
  { country: `usa`, key: `Yonkers`, value: `yonkers` },
];

async function city_country(tp, country_value) {
  const filtered_city_obj_arr = city_obj_arr.filter(
    (c) => c.country == `null` || c.country == country_value
  );
  const city_obj = await tp.system.suggester(
    (item) => item.key,
    filtered_city_obj_arr,
    false,
    "City?"
  );

  return city_obj;
}

module.exports = city_country;

```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET CITY
//---------------------------------------------------------  
const city = await tp.user.suggester_location(tp, country_value);
const city_name = city.key;
const city_value = city.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[country|Country Suggester]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[city_country.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[city suggester|City Suggester]]

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
