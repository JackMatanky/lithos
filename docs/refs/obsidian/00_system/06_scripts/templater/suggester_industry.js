let industry_obj_arr = [
  {
    key: "Abrasives and Nonmetallic Minerals Manufacturing",
    value: "abrasives_and_nonmetallic_minerals_manufacturing",
  },
  { key: "Accommodation", value: "accommodation" },
  { key: "Accounting", value: "accounting" },
  { key: "Administration of Justice", value: "administration_of_justice" },
  {
    key: "Administrative and Support Services",
    value: "administrative_and_support_services",
  },
  { key: "Advertising Services", value: "advertising_services" },
  {
    key: "Agricultural Chemical Manufacturing",
    value: "agricultural_chemical_manufacturing",
  },
  {
    key: "Agriculture, Construction, Mining Machinery Manufacturing",
    value: "agriculture_construction_mining_machinery_manufacturing",
  },
  {
    key: "Air, Water, and Waste Program Management",
    value: "air_water_and_waste_program_management",
  },
  { key: "Airlines and Aviation", value: "airlines_and_aviation" },
  {
    key: "Alternative Dispute Resolution",
    value: "alternative_dispute_resolution",
  },
  { key: "Alternative Medicine", value: "alternative_medicine" },
  { key: "Ambulance Services", value: "ambulance_services" },
  { key: "Amusement Parks and Arcades", value: "amusement_parks_and_arcades" },
  { key: "Animal Feed Manufacturing", value: "animal_feed_manufacturing" },
  {
    key: "Animation and Post-production",
    value: "animation_and_post_production",
  },
  { key: "Apparel Manufacturing", value: "apparel_manufacturing" },
  {
    key: "Appliances, Electrical, and Electronics Manufacturing",
    value: "appliances_electrical_and_electronics_manufacturing",
  },
  {
    key: "Architectural and Structural Metal Manufacturing",
    value: "architectural_and_structural_metal_manufacturing",
  },
  { key: "Architecture and Planning", value: "architecture_and_planning" },
  { key: "Armed Forces", value: "armed_forces" },
  {
    key: "Artificial Rubber and Synthetic Fiber Manufacturing",
    value: "artificial_rubber_and_synthetic_fiber_manufacturing",
  },
  { key: "Artists and Writers", value: "artists_and_writers" },
  {
    key: "Audio and Video Equipment Manufacturing",
    value: "audio_and_video_equipment_manufacturing",
  },
  {
    key: "Automation Machinery Manufacturing",
    value: "automation_machinery_manufacturing",
  },
  {
    key: "Aviation and Aerospace Component Manufacturing",
    value: "aviation_and_aerospace_component_manufacturing",
  },
  { key: "Baked Goods Manufacturing", value: "baked_goods_manufacturing" },
  { key: "Banking", value: "banking" },
  {
    key: "Bars, Taverns, and Nightclubs",
    value: "bars_taverns_and_nightclubs",
  },
  {
    key: "Bed-and-Breakfasts, Hostels, Homestays",
    value: "bed_and_breakfasts_hostels_homestays",
  },
  { key: "Beverage Manufacturing", value: "beverage_manufacturing" },
  {
    key: "Biomass Electric Power Generation",
    value: "biomass_electric_power_generation",
  },
  { key: "Biotechnology Research", value: "biotechnology_research" },
  { key: "Blockchain Services", value: "blockchain_services" },
  { key: "Blogs", value: "blogs" },
  {
    key: "Boilers, Tanks, and Shipping Container Manufacturing",
    value: "boilers_tanks_and_shipping_container_manufacturing",
  },
  {
    key: "Book and Periodical Publishing",
    value: "book_and_periodical_publishing",
  },
  { key: "Book Publishing", value: "book_publishing" },
  { key: "Breweries", value: "breweries" },
  {
    key: "Broadcast Media Production and Distribution",
    value: "broadcast_media_production_and_distribution",
  },
  { key: "Building Construction", value: "building_construction" },
  {
    key: "Building Equipment Contractors",
    value: "building_equipment_contractors",
  },
  {
    key: "Building Finishing Contractors",
    value: "building_finishing_contractors",
  },
  {
    key: "Building Structure and Exterior Contractors",
    value: "building_structure_and_exterior_contractors",
  },
  {
    key: "Business Consulting and Services",
    value: "business_consulting_and_services",
  },
  { key: "Business Content", value: "business_content" },
  {
    key: "Business Intelligence Platforms",
    value: "business_intelligence_platforms",
  },
  {
    key: "Cable and Satellite Programming",
    value: "cable_and_satellite_programming",
  },
  { key: "Capital Markets", value: "capital_markets" },
  { key: "Caterers", value: "caterers" },
  { key: "Chemical Manufacturing", value: "chemical_manufacturing" },
  {
    key: "Chemical Raw Materials Manufacturing",
    value: "chemical_raw_materials_manufacturing",
  },
  { key: "Child Day Care Services", value: "child_day_care_services" },
  { key: "Chiropractors", value: "chiropractors" },
  { key: "Circuses and Magic Shows", value: "circuses_and_magic_shows" },
  {
    key: "Civic and Social Organizations",
    value: "civic_and_social_organizations",
  },
  { key: "Civil Engineering", value: "civil_engineering" },
  {
    key: "Claims Adjusting, Actuarial Services",
    value: "claims_adjusting_actuarial_services",
  },
  {
    key: "Clay and Refractory Products Manufacturing",
    value: "clay_and_refractory_products_manufacturing",
  },
  { key: "Coal Mining", value: "coal_mining" },
  { key: "Collection Agencies", value: "collection_agencies" },
  {
    key: "Commercial and Industrial Equipment Rental",
    value: "commercial_and_industrial_equipment_rental",
  },
  {
    key: "Commercial and Industrial Machinery Maintenance",
    value: "commercial_and_industrial_machinery_maintenance",
  },
  {
    key: "Commercial and Service Industry Machinery Manufacturing",
    value: "commercial_and_service_industry_machinery_manufacturing",
  },
  {
    key: "Communications Equipment Manufacturing",
    value: "communications_equipment_manufacturing",
  },
  {
    key: "Community Development and Urban Planning",
    value: "community_development_and_urban_planning",
  },
  { key: "Community Services", value: "community_services" },
  {
    key: "Computer and Network Security",
    value: "computer_and_network_security",
  },
  { key: "Computer Games", value: "computer_games" },
  {
    key: "Computer Hardware Manufacturing",
    value: "computer_hardware_manufacturing",
  },
  {
    key: "Computer Networking Products",
    value: "computer_networking_products",
  },
  {
    key: "Computers and Electronics Manufacturing",
    value: "computers_and_electronics_manufacturing",
  },
  { key: "Conservation Programs", value: "conservation_programs" },
  { key: "Construction", value: "construction" },
  {
    key: "Construction Hardware Manufacturing",
    value: "construction_hardware_manufacturing",
  },
  { key: "Consumer Goods Rental", value: "consumer_goods_rental" },
  { key: "Consumer Services", value: "consumer_services" },
  { key: "Correctional Institutions", value: "correctional_institutions" },
  {
    key: "Cosmetology and Barber Schools",
    value: "cosmetology_and_barber_schools",
  },
  { key: "Courts of Law", value: "courts_of_law" },
  { key: "Credit Intermediation", value: "credit_intermediation" },
  {
    key: "Cutlery and Handtool Manufacturing",
    value: "cutlery_and_handtool_manufacturing",
  },
  { key: "Dairy Product Manufacturing", value: "dairy_product_manufacturing" },
  { key: "Dance Companies", value: "dance_companies" },
  {
    key: "Data Infrastructure and Analytics",
    value: "data_infrastructure_and_analytics",
  },
  {
    key: "Data Security Software Products",
    value: "data_security_software_products",
  },
  {
    key: "Defense and Space Manufacturing",
    value: "defense_and_space_manufacturing",
  },
  { key: "Dentists", value: "dentists" },
  { key: "Design Services", value: "design_services" },
  {
    key: "Desktop Computing Software Products",
    value: "desktop_computing_software_products",
  },
  { key: "Distilleries", value: "distilleries" },
  { key: "E-Learning Providers", value: "e_learning_providers" },
  { key: "Economic Programs", value: "economic_programs" },
  { key: "Education", value: "education" },
  {
    key: "Education Administration Programs",
    value: "education_administration_programs",
  },
  { key: "Education Management", value: "education_management" },
  {
    key: "Electric Lighting Equipment Manufacturing",
    value: "electric_lighting_equipment_manufacturing",
  },
  { key: "Electric Power Generation", value: "electric_power_generation" },
  {
    key: "Electric Power Transmission, Control, and Distribution",
    value: "electric_power_transmission_control_and_distribution",
  },
  {
    key: "Electrical Equipment Manufacturing",
    value: "electrical_equipment_manufacturing",
  },
  {
    key: "Electronic and Precision Equipment Maintenance",
    value: "electronic_and_precision_equipment_maintenance",
  },
  { key: "Embedded Software Products", value: "embedded_software_products" },
  {
    key: "Emergency and Relief Services",
    value: "emergency_and_relief_services",
  },
  { key: "Engineering Services", value: "engineering_services" },
  {
    key: "Engines and Power Transmission Equipment Manufacturing",
    value: "engines_and_power_transmission_equipment_manufacturing",
  },
  { key: "Entertainment Providers", value: "entertainment_providers" },
  { key: "Entertainment", value: "entertainment" },
  {
    key: "Environmental Quality Programs",
    value: "environmental_quality_programs",
  },
  { key: "Environmental Services", value: "environmental_services" },
  { key: "Equipment Rental Services", value: "equipment_rental_services" },
  { key: "Events Services", value: "events_services" },
  { key: "Executive Offices", value: "executive_offices" },
  { key: "Executive Search Services", value: "executive_search_services" },
  { key: "Fabricated Metal Products", value: "fabricated_metal_products" },
  { key: "Facilities Services", value: "facilities_services" },
  { key: "Family Planning Centers", value: "family_planning_centers" },
  { key: "Farming", value: "farming" },
  { key: "Farming, Ranching, Forestry", value: "farming_ranching_forestry" },
  {
    key: "Fashion Accessories Manufacturing",
    value: "fashion_accessories_manufacturing",
  },
  { key: "Financial Services", value: "financial_services" },
  { key: "Fine Arts Schools", value: "fine_arts_schools" },
  { key: "Fisheries", value: "fisheries" },
  { key: "Flight Training", value: "flight_training" },
  {
    key: "Food and Beverage Manufacturing",
    value: "food_and_beverage_manufacturing",
  },
  { key: "Food and Beverage Retail", value: "food_and_beverage_retail" },
  { key: "Food and Beverage Services", value: "food_and_beverage_services" },
  { key: "Food and Beverages", value: "food_and_beverages" },
  {
    key: "Footwear and Leather Goods Repair",
    value: "footwear_and_leather_goods_repair",
  },
  { key: "Footwear Manufacturing", value: "footwear_manufacturing" },
  { key: "Forestry and Logging", value: "forestry_and_logging" },
  {
    key: "Fossil Fuel Electric Power Generation",
    value: "fossil_fuel_electric_power_generation",
  },
  {
    key: "Freight and Package Transportation",
    value: "freight_and_package_transportation",
  },
  {
    key: "Fruit and Vegetable Preserves Manufacturing",
    value: "fruit_and_vegetable_preserves_manufacturing",
  },
  { key: "Fundraising", value: "fundraising" },
  { key: "Funds and Trusts", value: "funds_and_trusts" },
  {
    key: "Furniture and Home Furnishings Manufacturing",
    value: "furniture_and_home_furnishings_manufacturing",
  },
  {
    key: "Gambling Facilities and Casinos",
    value: "gambling_facilities_and_casinos",
  },
  {
    key: "Geothermal Electric Power Generation",
    value: "geothermal_electric_power_generation",
  },
  { key: "Glass Product Manufacturing", value: "glass_product_manufacturing" },
  {
    key: "Glass, Ceramics and Concrete Manufacturing",
    value: "glass_ceramics_and_concrete_manufacturing",
  },
  {
    key: "Golf Courses and Country Clubs",
    value: "golf_courses_and_country_clubs",
  },
  { key: "Government Administration", value: "government_administration" },
  {
    key: "Government Relations Services",
    value: "government_relations_services",
  },
  { key: "Graphic Design", value: "graphic_design" },
  {
    key: "Ground Passenger Transportation",
    value: "ground_passenger_transportation",
  },
  { key: "Health and Human Services", value: "health_and_human_services" },
  { key: "Higher Education", value: "higher_education" },
  {
    key: "Highway, Street, and Bridge Construction",
    value: "highway_street_and_bridge_construction",
  },
  { key: "Historical Sites", value: "historical_sites" },
  { key: "Holding Companies", value: "holding_companies" },
  { key: "Home Health Care Services", value: "home_health_care_services" },
  { key: "Horticulture", value: "horticulture" },
  { key: "Hospitality", value: "hospitality" },
  { key: "Hospitals", value: "hospitals" },
  { key: "Hospitals and Health Care", value: "hospitals_and_health_care" },
  { key: "Hotels and Motels", value: "hotels_and_motels" },
  {
    key: "Household and Institutional Furniture Manufacturing",
    value: "household_and_institutional_furniture_manufacturing",
  },
  {
    key: "Household Appliance Manufacturing",
    value: "household_appliance_manufacturing",
  },
  { key: "Household Services", value: "household_services" },
  {
    key: "Housing and Community Development",
    value: "housing_and_community_development",
  },
  { key: "Housing Programs", value: "housing_programs" },
  { key: "Human Resources Services", value: "human_resources_services" },
  {
    key: "HVAC and Refrigeration Equipment Manufacturing",
    value: "hvac_and_refrigeration_equipment_manufacturing",
  },
  {
    key: "Hydroelectric Power Generation",
    value: "hydroelectric_power_generation",
  },
  {
    key: "Individual and Family Services",
    value: "individual_and_family_services",
  },
  {
    key: "Industrial Machinery Manufacturing",
    value: "industrial_machinery_manufacturing",
  },
  { key: "Industry Associations", value: "industry_associations" },
  { key: "Information Services", value: "information_services" },
  {
    key: "Information Technology and Services",
    value: "information_technology_and_services",
  },
  { key: "Insurance", value: "insurance" },
  {
    key: "Insurance Agencies and Brokerages",
    value: "insurance_agencies_and_brokerages",
  },
  {
    key: "Insurance and Employee Benefit Funds",
    value: "insurance_and_employee_benefit_funds",
  },
  { key: "Insurance Carriers", value: "insurance_carriers" },
  { key: "Interior Design", value: "interior_design" },
  { key: "International Affairs", value: "international_affairs" },
  {
    key: "International Trade and Development",
    value: "international_trade_and_development",
  },
  {
    key: "Internet Marketplace Platforms",
    value: "internet_marketplace_platforms",
  },
  { key: "Internet News", value: "internet_news" },
  { key: "Internet Publishing", value: "internet_publishing" },
  {
    key: "Interurban and Rural Bus Services",
    value: "interurban_and_rural_bus_services",
  },
  { key: "Investment Advice", value: "investment_advice" },
  { key: "Investment Banking", value: "investment_banking" },
  { key: "Investment Management", value: "investment_management" },
  {
    key: "IT Services and IT Consulting",
    value: "it_services_and_it_consulting",
  },
  {
    key: "IT System Custom Software Development",
    value: "it_system_custom_software_development",
  },
  { key: "IT System Data Services", value: "it_system_data_services" },
  { key: "IT System Design Services", value: "it_system_design_services" },
  {
    key: "IT System Installation and Disposal",
    value: "it_system_installation_and_disposal",
  },
  {
    key: "IT System Operations and Maintenance",
    value: "it_system_operations_and_maintenance",
  },
  {
    key: "IT System Testing and Evaluation",
    value: "it_system_testing_and_evaluation",
  },
  {
    key: "IT System Training and Support",
    value: "it_system_training_and_support",
  },
  { key: "Janitorial Services", value: "janitorial_services" },
  { key: "Landscaping Services", value: "landscaping_services" },
  { key: "Language Schools", value: "language_schools" },
  {
    key: "Laundry and Drycleaning Services",
    value: "laundry_and_drycleaning_services",
  },
  { key: "Law Enforcement", value: "law_enforcement" },
  { key: "Law Practice", value: "law_practice" },
  {
    key: "Leasing Non-residential Real Estate",
    value: "leasing_non_residential_real_estate",
  },
  {
    key: "Leasing Residential Real Estate",
    value: "leasing_residential_real_estate",
  },
  {
    key: "Leather Product Manufacturing",
    value: "leather_product_manufacturing",
  },
  { key: "Legal Services", value: "legal_services" },
  { key: "Legislative Offices", value: "legislative_offices" },
  { key: "Leisure, Travel and Tourism", value: "leisure_travel_and_tourism" },
  { key: "Libraries", value: "libraries" },
  {
    key: "Lime and Gypsum Products Manufacturing",
    value: "lime_and_gypsum_products_manufacturing",
  },
  { key: "Loan Brokers", value: "loan_brokers" },
  { key: "Machinery Manufacturing", value: "machinery_manufacturing" },
  {
    key: "Magnetic and Optical Media Manufacturing",
    value: "magnetic_and_optical_media_manufacturing",
  },
  { key: "Manufacturing", value: "manufacturing" },
  { key: "Maritime Transportation", value: "maritime_transportation" },
  { key: "Market Research", value: "market_research" },
  { key: "Marketing Services", value: "marketing_services" },
  {
    key: "Mattress and Blinds Manufacturing",
    value: "mattress_and_blinds_manufacturing",
  },
  {
    key: "Measuring and Control Instrument Manufacturing",
    value: "measuring_and_control_instrument_manufacturing",
  },
  { key: "Meat Products Manufacturing", value: "meat_products_manufacturing" },
  {
    key: "Media and Telecommunications",
    value: "media_and_telecommunications",
  },
  { key: "Media Production", value: "media_production" },
  {
    key: "Medical and Diagnostic Laboratories",
    value: "medical_and_diagnostic_laboratories",
  },
  {
    key: "Medical Equipment Manufacturing",
    value: "medical_equipment_manufacturing",
  },
  { key: "Medical Practices", value: "medical_practices" },
  { key: "Mental Health Care", value: "mental_health_care" },
  { key: "Metal Ore Mining", value: "metal_ore_mining" },
  { key: "Metal Treatments", value: "metal_treatments" },
  {
    key: "Metal Valve, Ball, and Roller Manufacturing",
    value: "metal_valve_ball_and_roller_manufacturing",
  },
  {
    key: "Metalworking Machinery Manufacturing",
    value: "metalworking_machinery_manufacturing",
  },
  {
    key: "Military and International Affairs",
    value: "military_and_international_affairs",
  },
  { key: "Mining", value: "mining" },
  {
    key: "Mobile Computing Software Products",
    value: "mobile_computing_software_products",
  },
  { key: "Mobile Food Services", value: "mobile_food_services" },
  { key: "Mobile Gaming Apps", value: "mobile_gaming_apps" },
  { key: "Motor Vehicle Manufacturing", value: "motor_vehicle_manufacturing" },
  {
    key: "Motor Vehicle Parts Manufacturing",
    value: "motor_vehicle_parts_manufacturing",
  },
  { key: "Movies and Sound Recording", value: "movies_and_sound_recording" },
  { key: "Movies, Videos, and Sound", value: "movies_videos_and_sound" },
  { key: "Museums", value: "museums" },
  {
    key: "Museums, Historical Sites, and Zoos",
    value: "museums_historical_sites_and_zoos",
  },
  { key: "Musicians", value: "musicians" },
  { key: "Nanotechnology Research", value: "nanotechnology_research" },
  { key: "Natural Gas Distribution", value: "natural_gas_distribution" },
  { key: "Natural Gas Extraction", value: "natural_gas_extraction" },
  { key: "Newspaper Publishing", value: "newspaper_publishing" },
  { key: "Non-profit Organizations", value: "non_profit_organizations" },
  { key: "Online Media", value: "online_media" },
  { key: "Nonmetallic Mineral Mining", value: "nonmetallic_mineral_mining" },
  {
    key: "Nonresidential Building Construction",
    value: "nonresidential_building_construction",
  },
  {
    key: "Nuclear Electric Power Generation",
    value: "nuclear_electric_power_generation",
  },
  {
    key: "Nursing Homes and Residential Care Facilities",
    value: "nursing_homes_and_residential_care_facilities",
  },
  { key: "Office Administration", value: "office_administration" },
  {
    key: "Office Furniture and Fixtures Manufacturing",
    value: "office_furniture_and_fixtures_manufacturing",
  },
  {
    key: "Oil and Coal Product Manufacturing",
    value: "oil_and_coal_product_manufacturing",
  },
  { key: "Oil and Gas", value: "oil_and_gas" },
  { key: "Oil Extraction", value: "oil_extraction" },
  { key: "Oil, Gas, and Mining", value: "oil_gas_and_mining" },
  {
    key: "Online and Mail Order Retail",
    value: "online_and_mail_order_retail",
  },
  {
    key: "Online Audio and Video Media",
    value: "online_audio_and_video_media",
  },
  { key: "Operations Consulting", value: "operations_consulting" },
  { key: "Optometrists", value: "optometrists" },
  { key: "Outpatient Care Centers", value: "outpatient_care_centers" },
  {
    key: "Outsourcing and Offshoring Consulting",
    value: "outsourcing_and_offshoring_consulting",
  },
  {
    key: "Packaging and Containers Manufacturing",
    value: "packaging_and_containers_manufacturing",
  },
  {
    key: "Paint, Coating, and Adhesive Manufacturing",
    value: "paint_coating_and_adhesive_manufacturing",
  },
  {
    key: "Paper and Forest Product Manufacturing",
    value: "paper_and_forest_product_manufacturing",
  },
  { key: "Pension Funds", value: "pension_funds" },
  { key: "Performing Arts", value: "performing_arts" },
  {
    key: "Performing Arts and Spectator Sports",
    value: "performing_arts_and_spectator_sports",
  },
  { key: "Periodical Publishing", value: "periodical_publishing" },
  {
    key: "Personal and Laundry Services",
    value: "personal_and_laundry_services",
  },
  {
    key: "Personal Care Product Manufacturing",
    value: "personal_care_product_manufacturing",
  },
  { key: "Personal Care Services", value: "personal_care_services" },
  { key: "Pet Services", value: "pet_services" },
  {
    key: "Pharmaceutical Manufacturing",
    value: "pharmaceutical_manufacturing",
  },
  {
    key: "Philanthropic Fundraising Services",
    value: "philanthropic_fundraising_services",
  },
  { key: "Photography", value: "photography" },
  {
    key: "Physical, Occupational and Speech Therapists",
    value: "physical_occupational_and_speech_therapists",
  },
  { key: "Physicians", value: "physicians" },
  { key: "Pipeline Transportation", value: "pipeline_transportation" },
  {
    key: "Plastics and Rubber Product Manufacturing",
    value: "plastics_and_rubber_product_manufacturing",
  },
  { key: "Plastics Manufacturing", value: "plastics_manufacturing" },
  { key: "Political Organizations", value: "political_organizations" },
  { key: "Postal Services", value: "postal_services" },
  {
    key: "Primary and Secondary Education",
    value: "primary_and_secondary_education",
  },
  { key: "Primary Metal Manufacturing", value: "primary_metal_manufacturing" },
  { key: "Printing Services", value: "printing_services" },
  { key: "Professional Organizations", value: "professional_organizations" },
  { key: "Professional Services", value: "professional_services" },
  {
    key: "Professional Training and Coaching",
    value: "professional_training_and_coaching",
  },
  { key: "Public Assistance Programs", value: "public_assistance_programs" },
  { key: "Public Health", value: "public_health" },
  { key: "Public Policy Offices", value: "public_policy_offices" },
  {
    key: "Public Relations and Communications Services",
    value: "public_relations_and_communications_services",
  },
  { key: "Public Safety", value: "public_safety" },
  { key: "Racetracks", value: "racetracks" },
  {
    key: "Radio and Television Broadcasting",
    value: "radio_and_television_broadcasting",
  },
  { key: "Rail Transportation", value: "rail_transportation" },
  {
    key: "Railroad Equipment Manufacturing",
    value: "railroad_equipment_manufacturing",
  },
  { key: "Ranching", value: "ranching" },
  { key: "Ranching and Fisheries", value: "ranching_and_fisheries" },
  { key: "Real Estate", value: "real_estate" },
  {
    key: "Real Estate Agents and Brokers",
    value: "real_estate_agents_and_brokers",
  },
  {
    key: "Real Estate and Equipment Rental Services",
    value: "real_estate_and_equipment_rental_services",
  },
  { key: "Recreational Facilities", value: "recreational_facilities" },
  { key: "Religious Institutions", value: "religious_institutions" },
  {
    key: "Renewable Energy Equipment Manufacturing",
    value: "renewable_energy_equipment_manufacturing",
  },
  {
    key: "Renewable Energy Power Generation",
    value: "renewable_energy_power_generation",
  },
  {
    key: "Renewable Energy Semiconductor Manufacturing",
    value: "renewable_energy_semiconductor_manufacturing",
  },
  { key: "Repair and Maintenance", value: "repair_and_maintenance" },
  { key: "Research Services", value: "research_services" },
  { key: "Research", value: "research" },
  {
    key: "Residential Building Construction",
    value: "residential_building_construction",
  },
  { key: "Restaurants", value: "restaurants" },
  { key: "Retail", value: "retail" },
  { key: "Retail Apparel and Fashion", value: "retail_apparel_and_fashion" },
  {
    key: "Retail Appliances, Electrical, and Electronic Equipment",
    value: "retail_appliances_electrical_and_electronic_equipment",
  },
  { key: "Retail Art Dealers", value: "retail_art_dealers" },
  { key: "Retail Art Supplies", value: "retail_art_supplies" },
  {
    key: "Retail Books and Printed News",
    value: "retail_books_and_printed_news",
  },
  {
    key: "Retail Building Materials and Garden Equipment",
    value: "retail_building_materials_and_garden_equipment",
  },
  { key: "Retail Florists", value: "retail_florists" },
  {
    key: "Retail Furniture and Home Furnishings",
    value: "retail_furniture_and_home_furnishings",
  },
  { key: "Retail Gasoline", value: "retail_gasoline" },
  { key: "Retail Groceries", value: "retail_groceries" },
  {
    key: "Retail Health and Personal Care Products",
    value: "retail_health_and_personal_care_products",
  },
  {
    key: "Retail Luxury Goods and Jewelry",
    value: "retail_luxury_goods_and_jewelry",
  },
  { key: "Retail Motor Vehicles", value: "retail_motor_vehicles" },
  { key: "Retail Musical Instruments", value: "retail_musical_instruments" },
  { key: "Retail Office Equipment", value: "retail_office_equipment" },
  {
    key: "Retail Office Supplies and Gifts",
    value: "retail_office_supplies_and_gifts",
  },
  {
    key: "Retail Recyclable Materials and Used Merchandise",
    value: "retail_recyclable_materials_and_used_merchandise",
  },
  {
    key: "Reupholstery and Furniture Repair",
    value: "reupholstery_and_furniture_repair",
  },
  {
    key: "Rubber Products Manufacturing",
    value: "rubber_products_manufacturing",
  },
  {
    key: "Satellite Telecommunications",
    value: "satellite_telecommunications",
  },
  { key: "Savings Institutions", value: "savings_institutions" },
  {
    key: "School and Employee Bus Services",
    value: "school_and_employee_bus_services",
  },
  {
    key: "Seafood Product Manufacturing",
    value: "seafood_product_manufacturing",
  },
  { key: "Secretarial Schools", value: "secretarial_schools" },
  {
    key: "Securities and Commodity Exchanges",
    value: "securities_and_commodity_exchanges",
  },
  { key: "Security and Investigations", value: "security_and_investigations" },
  {
    key: "Security Guards and Patrol Services",
    value: "security_guards_and_patrol_services",
  },
  { key: "Security Systems Services", value: "security_systems_services" },
  { key: "Semiconductor Manufacturing", value: "semiconductor_manufacturing" },
  {
    key: "Services for Renewable Energy",
    value: "services_for_renewable_energy",
  },
  {
    key: "Services for the Elderly and Disabled",
    value: "services_for_the_elderly_and_disabled",
  },
  { key: "Sheet Music Publishing", value: "sheet_music_publishing" },
  { key: "Shipbuilding", value: "shipbuilding" },
  {
    key: "Shuttles and Special Needs Transportation Services",
    value: "shuttles_and_special_needs_transportation_services",
  },
  { key: "Sightseeing Transportation", value: "sightseeing_transportation" },
  { key: "Skiing Facilities", value: "skiing_facilities" },
  {
    key: "Soap and Cleaning Product Manufacturing",
    value: "soap_and_cleaning_product_manufacturing",
  },
  { key: "Social Networking Platforms", value: "social_networking_platforms" },
  { key: "Software Development", value: "software_development" },
  {
    key: "Solar Electric Power Generation",
    value: "solar_electric_power_generation",
  },
  { key: "Sound Recording", value: "sound_recording" },
  {
    key: "Space Research and Technology",
    value: "space_research_and_technology",
  },
  { key: "Specialty Trade Contractors", value: "specialty_trade_contractors" },
  { key: "Spectator Sports", value: "spectator_sports" },
  {
    key: "Sporting Goods Manufacturing",
    value: "sporting_goods_manufacturing",
  },
  {
    key: "Sports and Recreation Instruction",
    value: "sports_and_recreation_instruction",
  },
  { key: "Sports Teams and Clubs", value: "sports_teams_and_clubs" },
  {
    key: "Spring and Wire Product Manufacturing",
    value: "spring_and_wire_product_manufacturing",
  },
  { key: "Staffing and Recruiting", value: "staffing_and_recruiting" },
  {
    key: "Steam and Air-Conditioning Supply",
    value: "steam_and_air_conditioning_supply",
  },
  {
    key: "Strategic Management Services",
    value: "strategic_management_services",
  },
  { key: "Subdivision of Land", value: "subdivision_of_land" },
  {
    key: "Sugar and Confectionery Product Manufacturing",
    value: "sugar_and_confectionery_product_manufacturing",
  },
  { key: "Taxi and Limousine Services", value: "taxi_and_limousine_services" },
  {
    key: "Technical and Vocational Training",
    value: "technical_and_vocational_training",
  },
  {
    key: "Technology, Information and Internet",
    value: "technology_information_and_internet",
  },
  {
    key: "Technology, Information and Media",
    value: "technology_information_and_media",
  },
  { key: "Telecommunications", value: "telecommunications" },
  { key: "Telecommunications Carriers", value: "telecommunications_carriers" },
  { key: "Telephone Call Centers", value: "telephone_call_centers" },
  { key: "Temporary Help Services", value: "temporary_help_services" },
  { key: "Textile Manufacturing", value: "textile_manufacturing" },
  { key: "Theater Companies", value: "theater_companies" },
  { key: "Think Tanks", value: "think_tanks" },
  { key: "Tobacco Manufacturing", value: "tobacco_manufacturing" },
  {
    key: "Translation and Localization",
    value: "translation_and_localization",
  },
  {
    key: "Transportation Equipment Manufacturing",
    value: "transportation_equipment_manufacturing",
  },
  { key: "Transportation Programs", value: "transportation_programs" },
  {
    key: "Transportation, Logistics, Supply Chain and Storage",
    value: "transportation_logistics_supply_chain_and_storage",
  },
  { key: "Travel Arrangements", value: "travel_arrangements" },
  { key: "Truck Transportation", value: "truck_transportation" },
  { key: "Trusts and Estates", value: "trusts_and_estates" },
  {
    key: "Turned Products and Fastener Manufacturing",
    value: "turned_products_and_fastener_manufacturing",
  },
  { key: "Urban Transit Services", value: "urban_transit_services" },
  { key: "Utilities", value: "utilities" },
  { key: "Utilities Administration", value: "utilities_administration" },
  { key: "Utility System Construction", value: "utility_system_construction" },
  {
    key: "Vehicle Repair and Maintenance",
    value: "vehicle_repair_and_maintenance",
  },
  {
    key: "Venture Capital and Private Equity Principals",
    value: "venture_capital_and_private_equity_principals",
  },
  { key: "Veterinary Services", value: "veterinary_services" },
  {
    key: "Vocational Rehabilitation Services",
    value: "vocational_rehabilitation_services",
  },
  { key: "Warehousing and Storage", value: "warehousing_and_storage" },
  { key: "Waste Collection", value: "waste_collection" },
  {
    key: "Waste Treatment and Disposal",
    value: "waste_treatment_and_disposal",
  },
  {
    key: "Water Supply and Irrigation Systems",
    value: "water_supply_and_irrigation_systems",
  },
  {
    key: "Water, Waste, Steam, and Air Conditioning Services",
    value: "water_waste_steam_and_air_conditioning_services",
  },
  {
    key: "Wellness and Fitness Services",
    value: "wellness_and_fitness_services",
  },
  { key: "Wholesale", value: "wholesale" },
  {
    key: "Wholesale Alcoholic Beverages",
    value: "wholesale_alcoholic_beverages",
  },
  {
    key: "Wholesale Apparel and Sewing Supplies",
    value: "wholesale_apparel_and_sewing_supplies",
  },
  {
    key: "Wholesale Appliances, Electrical, and Electronics",
    value: "wholesale_appliances_electrical_and_electronics",
  },
  {
    key: "Wholesale Building Materials",
    value: "wholesale_building_materials",
  },
  {
    key: "Wholesale Chemical and Allied Products",
    value: "wholesale_chemical_and_allied_products",
  },
  {
    key: "Wholesale Computer Equipment",
    value: "wholesale_computer_equipment",
  },
  {
    key: "Wholesale Drugs and Sundries",
    value: "wholesale_drugs_and_sundries",
  },
  { key: "Wholesale Food and Beverage", value: "wholesale_food_and_beverage" },
  { key: "Wholesale Footwear", value: "wholesale_footwear" },
  {
    key: "Wholesale Furniture and Home Furnishings",
    value: "wholesale_furniture_and_home_furnishings",
  },
  {
    key: "Wholesale Hardware, Plumbing, Heating Equipment",
    value: "wholesale_hardware_plumbing_heating_equipment",
  },
  { key: "Wholesale Import and Export", value: "wholesale_import_and_export" },
  {
    key: "Wholesale Luxury Goods and Jewelry",
    value: "wholesale_luxury_goods_and_jewelry",
  },
  { key: "Wholesale Machinery", value: "wholesale_machinery" },
  {
    key: "Wholesale Metals and Minerals",
    value: "wholesale_metals_and_minerals",
  },
  {
    key: "Wholesale Motor Vehicles and Parts",
    value: "wholesale_motor_vehicles_and_parts",
  },
  { key: "Wholesale Paper Products", value: "wholesale_paper_products" },
  {
    key: "Wholesale Petroleum and Petroleum Products",
    value: "wholesale_petroleum_and_petroleum_products",
  },
  {
    key: "Wholesale Photography Equipment and Supplies",
    value: "wholesale_photography_equipment_and_supplies",
  },
  { key: "Wholesale Raw Farm Products", value: "wholesale_raw_farm_products" },
  {
    key: "Wholesale Recyclable Materials",
    value: "wholesale_recyclable_materials",
  },
  {
    key: "Wind Electric Power Generation",
    value: "wind_electric_power_generation",
  },
  { key: "Wineries", value: "wineries" },
  { key: "Wireless Services", value: "wireless_services" },
  {
    key: "Women's Handbag Manufacturing",
    value: "women's_handbag_manufacturing",
  },
  { key: "Wood Product Manufacturing", value: "wood_product_manufacturing" },
  { key: "Writing and Editing", value: "writing_and_editing" },
  { key: "Zoos and Botanical Gardens", value: "zoos_and_botanical_gardens" },
];

const obj_arr = [
  { key: "Null", value: "null" },
  { key: "User Input", value: "_user_input" },
];

obj_arr.push(industry_obj_arr);

industry_obj_arr = obj_arr.flat();

async function suggester_industry(tp) {
  let industry_obj = await tp.system.suggester(
    (item) => item.key,
    industry_obj_arr,
    false,
    "LinkedIn Industry?"
  );

  let industry_name = industry_obj.key;
  let industry_value = industry_obj.value;

  if (industry_value == "_user_input") {
    industry_name = await tp.system.prompt("LinkedIn Industry?");
    industry_value = industry_name
      .replaceAll(/,/g, "")
      .replaceAll(/\s/g, "_")
      .replaceAll(/\//g, "-")
      .replaceAll(/&/g, "and")
      .toLowerCase();
  }

  industry_obj = {
    key: industry_name,
    value: industry_value,
  };

  return industry_obj;
}

module.exports = suggester_industry;
