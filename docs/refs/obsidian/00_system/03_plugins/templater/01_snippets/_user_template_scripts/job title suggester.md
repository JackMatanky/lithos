---
title: job title suggester
aliases:
  - Job Title Suggester
  - job_title_suggester
  - job_title
plugin: templater
language:
  - javascript
module:
  - user
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-01T09:36
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Job Title Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a job title from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
let job_titles_obj_arr = [
  { key: `Account Collector`, value: `account_collector` },
  { key: `Account Executive`, value: `account_executive` },
  { key: `Account Manager`, value: `account_manager` },
  { key: `Account Representative`, value: `account_representative` },
  { key: `Accountant`, value: `accountant` },
  { key: `Accounting Analyst`, value: `accounting_analyst` },
  { key: `Accounting Director`, value: `accounting_director` },
  { key: `Acquisitions Editor`, value: `acquisitions_editor` },
  { key: `Administrative Analyst`, value: `administrative_analyst` },
  { key: `Administrative Assistant`, value: `administrative_assistant` },
  { key: `Administrative Manager`, value: `administrative_manager` },
  { key: `Administrative Specialist`, value: `administrative_specialist` },
  { key: `Administrator`, value: `administrator` },
  { key: `Advertising Manager`, value: `advertising_manager` },
  { key: `Advisor`, value: `advisor` },
  { key: `Aerospace Engineer`, value: `aerospace_engineer` },
  { key: `Agent`, value: `agent` },
  { key: `Agricultural Engineer`, value: `agricultural_engineer` },
  { key: `Agricultural Scientist`, value: `agricultural_scientist` },
  { key: `AI/ML Engineer`, value: `ai_ml_engineer` },
  { key: `Aide`, value: `aide` },
  { key: `Air Traffic Controller`, value: `air_traffic_controller` },
  { key: `Aircraft Mechanic`, value: `aircraft_mechanic` },
  { key: `Analyst`, value: `analyst` },
  { key: `Animator`, value: `animator` },
  { key: `App Developer`, value: `app_developer` },
  { key: `Application Developer`, value: `application_developer` },
  { key: `Architect`, value: `architect` },
  { key: `Archivist`, value: `archivist` },
  { key: `Area Sales Manager`, value: `area_sales_manager` },
  { key: `Art Director`, value: `art_director` },
  {
    key: `Artificial Intelligence Engineer`,
    value: `artificial_intelligence_engineer`,
  },
  {
    key: `Artificial Intelligence Specialist`,
    value: `artificial_intelligence_specialist`,
  },
  { key: `Artist`, value: `artist` },
  { key: `Assistant`, value: `assistant` },
  { key: `Assistant Editor`, value: `assistant_editor` },
  { key: `Assistant Engineer`, value: `assistant_engineer` },
  { key: `Assistant Manager`, value: `assistant_manager` },
  { key: `Assistant Professor`, value: `assistant_professor` },
  { key: `Associate Editor`, value: `associate_editor` },
  { key: `Astronomer`, value: `astronomer` },
  { key: `Atmospheric Scientist`, value: `atmospheric_scientist` },
  { key: `Attendant`, value: `attendant` },
  { key: `Attorney`, value: `attorney` },
  { key: `Audio Engineer`, value: `audio_engineer` },
  { key: `Audiovisual Specialist`, value: `audiovisual_specialist` },
  { key: `Auditing Clerk`, value: `auditing_clerk` },
  { key: `Auditor`, value: `auditor` },
  { key: `Auto Mechanic`, value: `auto_mechanic` },
  { key: `Automotive Designer`, value: `automotive_designer` },
  { key: `B2B Sales Specialist`, value: `b2b_sales_specialist` },
  { key: `Back-end Developer`, value: `back-end_developer` },
  { key: `Baker`, value: `baker` },
  { key: `Bank Teller`, value: `bank_teller` },
  { key: `Barber`, value: `barber` },
  { key: `Barista`, value: `barista` },
  { key: `Bartender`, value: `bartender` },
  { key: `Benefits Coordinator`, value: `benefits_coordinator` },
  { key: `Benefits Manager`, value: `benefits_manager` },
  { key: `Big Data Engineer`, value: `big_data_engineer` },
  { key: `Biological Engineer`, value: `biological_engineer` },
  { key: `Biologist`, value: `biologist` },
  { key: `Biomedical Engineer`, value: `biomedical_engineer` },
  { key: `Biostatistician`, value: `biostatistician` },
  { key: `Boilermaker`, value: `boilermaker` },
  { key: `Bookkeeper`, value: `bookkeeper` },
  { key: `Branch Manager`, value: `branch_manager` },
  { key: `Brand Manager`, value: `brand_manager` },
  { key: `Brand Strategist`, value: `brand_strategist` },
  { key: `Broker`, value: `broker` },
  { key: `Budget Analyst`, value: `budget_analyst` },
  { key: `Building Inspector`, value: `building_inspector` },
  { key: `Business Analyst`, value: `business_analyst` },
  {
    key: `Business Development Manager`,
    value: `business_development_manager`,
  },
  {
    key: `Business Intelligence Analyst`,
    value: `business_intelligence_analyst`,
  },
  {
    key: `Business Intelligence Consultant`,
    value: `business_intelligence_consultant`,
  },
  {
    key: `Business Intelligence Developer`,
    value: `business_intelligence_developer`,
  },
  {
    key: `Business Intelligence Director`,
    value: `business_intelligence_director`,
  },
  {
    key: `Business Intelligence Intern`,
    value: `business_intelligence_intern`,
  },
  { key: `Business Intelligence Lead`, value: `business_intelligence_lead` },
  {
    key: `Business Intelligence Manager`,
    value: `business_intelligence_manager`,
  },
  {
    key: `Business Intelligence Specialist`,
    value: `business_intelligence_specialist`,
  },
  { key: `Business Manager`, value: `business_manager` },
  { key: `Business Operations Manager`, value: `business_operations_manager` },
  {
    key: `Business Transformation Manager`,
    value: `business_transformation_manager`,
  },
  { key: `Buyer`, value: `buyer` },
  { key: `Call Center Representative`, value: `call_center_representative` },
  { key: `Camera Operator`, value: `camera_operator` },
  { key: `Caregiver`, value: `caregiver` },
  { key: `Carpenter`, value: `carpenter` },
  { key: `Cashier`, value: `cashier` },
  { key: `Chef`, value: `chef` },
  { key: `Chemical Engineer`, value: `chemical_engineer` },
  { key: `Chemist`, value: `chemist` },
  { key: `Chief Architect`, value: `chief_architect` },
  { key: `Chief Customer Officer`, value: `chief_customer_officer` },
  { key: `Chief Data Officer`, value: `chief_data_officer` },
  { key: `Chief Engineer`, value: `chief_engineer` },
  { key: `Chief Executive Officer`, value: `chief_executive_officer` },
  { key: `Chief Financial Officer`, value: `chief_financial_officer` },
  {
    key: `Chief Human Resources Officer`,
    value: `chief_human_resources_officer`,
  },
  { key: `Chief Information Officer`, value: `chief_information_officer` },
  {
    key: `Chief Information Security Officer`,
    value: `chief_information_security_officer`,
  },
  { key: `Chief Marketing Officer`, value: `chief_marketing_officer` },
  { key: `Chief Operating Officer`, value: `chief_operating_officer` },
  { key: `Chief People Officer`, value: `chief_people_officer` },
  { key: `Chief Product Officer`, value: `chief_product_officer` },
  { key: `Chief Security Officer`, value: `chief_security_officer` },
  { key: `Chief Technology Officer`, value: `chief_technology_officer` },
  { key: `Civil Engineer`, value: `civil_engineer` },
  { key: `Claims Adjuster`, value: `claims_adjuster` },
  { key: `Clerk`, value: `clerk` },
  { key: `Client Partner`, value: `client_partner` },
  { key: `Client Service Specialist`, value: `client_service_specialist` },
  { key: `Clinical Psychologist`, value: `clinical_psychologist` },
  { key: `Cloud Architect`, value: `cloud_architect` },
  { key: `Cloud Data Analyst`, value: `cloud_data_analyst` },
  { key: `Cloud Data Architect`, value: `cloud_data_architect` },
  { key: `Cloud Data Engineer`, value: `cloud_data_engineer` },
  { key: `Cloud Data Lead`, value: `cloud_data_lead` },
  { key: `Cloud Data Scientist`, value: `cloud_data_scientist` },
  { key: `Cloud Data Specialist`, value: `cloud_data_specialist` },
  { key: `Cloud Engineer`, value: `cloud_engineer` },
  { key: `CNA`, value: `cna` },
  { key: `Coach`, value: `coach` },
  { key: `Columnist`, value: `columnist` },
  { key: `Commercial Loan Officer`, value: `commercial_loan_officer` },
  { key: `Commercial Pilot`, value: `commercial_pilot` },
  { key: `Communications Director`, value: `communications_director` },
  { key: `Communications Manager`, value: `communications_manager` },
  { key: `Community Health Worker`, value: `community_health_worker` },
  { key: `Community Organizer`, value: `community_organizer` },
  { key: `Computer Hardware Engineer`, value: `computer_hardware_engineer` },
  { key: `Computer Network Architect`, value: `computer_network_architect` },
  { key: `Computer Programmer`, value: `computer_programmer` },
  { key: `Computer Scientist`, value: `computer_scientist` },
  { key: `Computer Systems Analyst`, value: `computer_systems_analyst` },
  { key: `Computer Technician`, value: `computer_technician` },
  { key: `Concierge`, value: `concierge` },
  { key: `Conservation Scientist`, value: `conservation_scientist` },
  { key: `Construction Manager`, value: `construction_manager` },
  { key: `Construction Worker`, value: `construction_worker` },
  { key: `Consultant`, value: `consultant` },
  { key: `Content Creator`, value: `content_creator` },
  { key: `Content Marketing Manager`, value: `content_marketing_manager` },
  { key: `Content Strategist`, value: `content_strategist` },
  {
    key: `Continuous Improvement Consultant`,
    value: `continuous_improvement_consultant`,
  },
  { key: `Continuous Improvement Lead`, value: `continuous_improvement_lead` },
  { key: `Contract Administrator`, value: `contract_administrator` },
  { key: `Controller`, value: `controller` },
  { key: `Coordinator`, value: `coordinator` },
  { key: `Copy Editor`, value: `copy_editor` },
  { key: `Copywriter`, value: `copywriter` },
  { key: `Corporate Attorney`, value: `corporate_attorney` },
  { key: `Corporate Trainer`, value: `corporate_trainer` },
  { key: `Cost Estimator`, value: `cost_estimator` },
  { key: `Counsel`, value: `counsel` },
  { key: `Counselor`, value: `counselor` },
  { key: `Crane Operator`, value: `crane_operator` },
  { key: `Creative Director`, value: `creative_director` },
  { key: `Credit Analyst`, value: `credit_analyst` },
  { key: `Credit Authorizer`, value: `credit_authorizer` },
  { key: `Credit Counselor`, value: `credit_counselor` },
  { key: `Crew`, value: `crew` },
  { key: `Curator`, value: `curator` },
  { key: `Customer Advocate`, value: `customer_advocate` },
  { key: `Customer Care Associate`, value: `customer_care_associate` },
  { key: `Customer Service`, value: `customer_service` },
  { key: `Customer Service Manager`, value: `customer_service_manager` },
  {
    key: `Customer Service Representative`,
    value: `customer_service_representative`,
  },
  { key: `Customer Success Manager`, value: `customer_success_manager` },
  { key: `Customer Support`, value: `customer_support` },
  { key: `Cybersecurity Analyst`, value: `cybersecurity_analyst` },
  { key: `Data Analyst`, value: `data_analyst` },
  { key: `Data Analyst Advisor`, value: `data_analyst_advisor` },
  { key: `Data Analyst Architect`, value: `data_analyst_architect` },
  { key: `Data Analyst Auditor`, value: `data_analyst_auditor` },
  {
    key: `Data Analyst Business Partner`,
    value: `data_analyst_business_partner`,
  },
  { key: `Data Analyst Coach`, value: `data_analyst_coach` },
  { key: `Data Analyst Consultant`, value: `data_analyst_consultant` },
  { key: `Data Analyst Coordinator`, value: `data_analyst_coordinator` },
  { key: `Data Analyst Director`, value: `data_analyst_director` },
  { key: `Data Analyst Educator`, value: `data_analyst_educator` },
  { key: `Data Analyst Facilitator`, value: `data_analyst_facilitator` },
  { key: `Data Analyst Intern`, value: `data_analyst_intern` },
  { key: `Data Analyst Lead`, value: `data_analyst_lead` },
  {
    key: `Data Analyst Lead Consultant`,
    value: `data_analyst_lead_consultant`,
  },
  { key: `Data Analyst Manager`, value: `data_analyst_manager` },
  {
    key: `Data Analyst Manager Consultant`,
    value: `data_analyst_manager_consultant`,
  },
  { key: `Data Analyst Officer`, value: `data_analyst_officer` },
  { key: `Data Analyst Planner`, value: `data_analyst_planner` },
  {
    key: `Data Analyst Process Specialist`,
    value: `data_analyst_process_specialist`,
  },
  {
    key: `Data Analyst Program Manager`,
    value: `data_analyst_program_manager`,
  },
  {
    key: `Data Analyst Project Manager`,
    value: `data_analyst_project_manager`,
  },
  { key: `Data Analyst Recruiter`, value: `data_analyst_recruiter` },
  {
    key: `Data Analyst Relationship Manager`,
    value: `data_analyst_relationship_manager`,
  },
  { key: `Data Analyst Researcher`, value: `data_analyst_researcher` },
  { key: `Data Analyst Specialist`, value: `data_analyst_specialist` },
  { key: `Data Analyst Strategist`, value: `data_analyst_strategist` },
  { key: `Data Analyst Supervisor`, value: `data_analyst_supervisor` },
  { key: `Data Analyst Support`, value: `data_analyst_support` },
  { key: `Data Analyst Surveyor`, value: `data_analyst_surveyor` },
  {
    key: `Data Analyst Systems Analyst`,
    value: `data_analyst_systems_analyst`,
  },
  { key: `Data Analyst Team Leader`, value: `data_analyst_team_leader` },
  {
    key: `Data Analyst Technical Specialist`,
    value: `data_analyst_technical_specialist`,
  },
  { key: `Data Analyst Trainer`, value: `data_analyst_trainer` },
  { key: `Data Analyst Trainer/Coach`, value: `data_analyst_trainer_coach` },
  {
    key: `Data Analyst Training Coordinator`,
    value: `data_analyst_training_coordinator`,
  },
  {
    key: `Data Analyst Validation Specialist`,
    value: `data_analyst_validation_specialist`,
  },
  {
    key: `Data Analyst Workflow Analyst`,
    value: `data_analyst_workflow_analyst`,
  },
  { key: `Data Analytics Architect`, value: `data_analytics_architect` },
  { key: `Data Analytics Associate`, value: `data_analytics_associate` },
  { key: `Data Analytics Auditor`, value: `data_analytics_auditor` },
  {
    key: `Data Analytics Business Partner`,
    value: `data_analytics_business_partner`,
  },
  { key: `Data Analytics Coach`, value: `data_analytics_coach` },
  { key: `Data Analytics Consultant`, value: `data_analytics_consultant` },
  { key: `Data Analytics Coordinator`, value: `data_analytics_coordinator` },
  { key: `Data Analytics Developer`, value: `data_analytics_developer` },
  { key: `Data Analytics Director`, value: `data_analytics_director` },
  { key: `Data Analytics Educator`, value: `data_analytics_educator` },
  { key: `Data Analytics Facilitator`, value: `data_analytics_facilitator` },
  { key: `Data Analytics Lead`, value: `data_analytics_lead` },
  { key: `Data Analytics Manager`, value: `data_analytics_manager` },
  { key: `Data Analytics Officer`, value: `data_analytics_officer` },
  { key: `Data Analytics Specialist`, value: `data_analytics_specialist` },
  { key: `Data Analytics Strategist`, value: `data_analytics_strategist` },
  { key: `Data Analytics Supervisor`, value: `data_analytics_supervisor` },
  { key: `Data Analytics Technician`, value: `data_analytics_technician` },
  { key: `Data Analytics Trainer`, value: `data_analytics_trainer` },
  { key: `Data Architect`, value: `data_architect` },
  { key: `Data Center Analyst`, value: `data_center_analyst` },
  { key: `Data Center Architect`, value: `data_center_architect` },
  { key: `Data Center Engineer`, value: `data_center_engineer` },
  { key: `Data Center Manager`, value: `data_center_manager` },
  { key: `Data Center Technician`, value: `data_center_technician` },
  { key: `Data Compliance Analyst`, value: `data_compliance_analyst` },
  { key: `Data Compliance Lead`, value: `data_compliance_lead` },
  { key: `Data Compliance Manager`, value: `data_compliance_manager` },
  { key: `Data Compliance Specialist`, value: `data_compliance_specialist` },
  { key: `Data Conversion Specialist`, value: `data_conversion_specialist` },
  { key: `Data Engineer`, value: `data_engineer` },
  { key: `Data Engineer Intern`, value: `data_engineer_intern` },
  { key: `Data Engineering Lead`, value: `data_engineering_lead` },
  { key: `Data Engineering Specialist`, value: `data_engineering_specialist` },
  { key: `Data Entry`, value: `data_entry` },
  { key: `Data Entry Clerk`, value: `data_entry_clerk` },
  { key: `Data Entry Specialist`, value: `data_entry_specialist` },
  {
    key: `Data Governance Administrator`,
    value: `data_governance_administrator`,
  },
  { key: `Data Governance Analyst`, value: `data_governance_analyst` },
  { key: `Data Governance Architect`, value: `data_governance_architect` },
  { key: `Data Governance Consultant`, value: `data_governance_consultant` },
  { key: `Data Governance Coordinator`, value: `data_governance_coordinator` },
  { key: `Data Governance Director`, value: `data_governance_director` },
  { key: `Data Governance Lead`, value: `data_governance_lead` },
  { key: `Data Governance Manager`, value: `data_governance_manager` },
  { key: `Data Governance Officer`, value: `data_governance_officer` },
  { key: `Data Governance Specialist`, value: `data_governance_specialist` },
  { key: `Data Integration Developer`, value: `data_integration_developer` },
  { key: `Data Integration Specialist`, value: `data_integration_specialist` },
  { key: `Data Management Analyst`, value: `data_management_analyst` },
  { key: `Data Management Consultant`, value: `data_management_consultant` },
  { key: `Data Management Coordinator`, value: `data_management_coordinator` },
  { key: `Data Management Director`, value: `data_management_director` },
  { key: `Data Management Engineer`, value: `data_management_engineer` },
  { key: `Data Management Lead`, value: `data_management_lead` },
  { key: `Data Management Manager`, value: `data_management_manager` },
  { key: `Data Management Specialist`, value: `data_management_specialist` },
  { key: `Data Migration Lead`, value: `data_migration_lead` },
  { key: `Data Migration Manager`, value: `data_migration_manager` },
  { key: `Data Migration Specialist`, value: `data_migration_specialist` },
  { key: `Data Mining Analyst`, value: `data_mining_analyst` },
  { key: `Data Mining Engineer`, value: `data_mining_engineer` },
  { key: `Data Mining Lead`, value: `data_mining_lead` },
  { key: `Data Mining Manager`, value: `data_mining_manager` },
  { key: `Data Mining Specialist`, value: `data_mining_specialist` },
  { key: `Data Modeler`, value: `data_modeler` },
  { key: `Data Operations Manager`, value: `data_operations_manager` },
  { key: `Data Privacy Analyst`, value: `data_privacy_analyst` },
  { key: `Data Privacy Lead`, value: `data_privacy_lead` },
  { key: `Data Privacy Manager`, value: `data_privacy_manager` },
  { key: `Data Privacy Officer`, value: `data_privacy_officer` },
  { key: `Data Privacy Specialist`, value: `data_privacy_specialist` },
  { key: `Data Protection Officer`, value: `data_protection_officer` },
  { key: `Data Quality Analyst`, value: `data_quality_analyst` },
  { key: `Data Quality Consultant`, value: `data_quality_consultant` },
  { key: `Data Quality Developer`, value: `data_quality_developer` },
  { key: `Data Quality Lead`, value: `data_quality_lead` },
  { key: `Data Quality Manager`, value: `data_quality_manager` },
  { key: `Data Quality Specialist`, value: `data_quality_specialist` },
  { key: `Data Science Analyst`, value: `data_science_analyst` },
  { key: `Data Science Consultant`, value: `data_science_consultant` },
  { key: `Data Science Developer`, value: `data_science_developer` },
  { key: `Data Science Director`, value: `data_science_director` },
  { key: `Data Science Intern`, value: `data_science_intern` },
  { key: `Data Science Lead`, value: `data_science_lead` },
  { key: `Data Science Manager`, value: `data_science_manager` },
  { key: `Data Science Specialist`, value: `data_science_specialist` },
  { key: `Data Scientist`, value: `data_scientist` },
  { key: `Data Scientist Consultant`, value: `data_scientist_consultant` },
  { key: `Data Security Analyst`, value: `data_security_analyst` },
  { key: `Data Security Lead`, value: `data_security_lead` },
  { key: `Data Security Manager`, value: `data_security_manager` },
  { key: `Data Security Specialist`, value: `data_security_specialist` },
  { key: `Data Steward`, value: `data_steward` },
  { key: `Data Storyteller`, value: `data_storyteller` },
  { key: `Data Strategy Lead`, value: `data_strategy_lead` },
  { key: `Data Strategy Manager`, value: `data_strategy_manager` },
  { key: `Data Strategy Specialist`, value: `data_strategy_specialist` },
  { key: `Data Visualization Analyst`, value: `data_visualization_analyst` },
  {
    key: `Data Visualization Developer`,
    value: `data_visualization_developer`,
  },
  { key: `Data Visualization Lead`, value: `data_visualization_lead` },
  { key: `Data Visualization Manager`, value: `data_visualization_manager` },
  {
    key: `Data Visualization Specialist`,
    value: `data_visualization_specialist`,
  },
  {
    key: `Data Warehouse Administrator`,
    value: `data_warehouse_administrator`,
  },
  { key: `Data Warehouse Analyst`, value: `data_warehouse_analyst` },
  { key: `Data Warehouse Architect`, value: `data_warehouse_architect` },
  { key: `Data Warehouse Consultant`, value: `data_warehouse_consultant` },
  { key: `Data Warehouse Developer`, value: `data_warehouse_developer` },
  { key: `Data Warehouse Engineer`, value: `data_warehouse_engineer` },
  { key: `Data Warehouse Lead`, value: `data_warehouse_lead` },
  { key: `Data Warehouse Manager`, value: `data_warehouse_manager` },
  { key: `Data Warehouse Specialist`, value: `data_warehouse_specialist` },
  { key: `Data Wrangler`, value: `data_wrangler` },
  { key: `Database Administrator`, value: `database_administrator` },
  {
    key: `Database Administrator Intern`,
    value: `database_administrator_intern`,
  },
  { key: `Dental Assistant`, value: `dental_assistant` },
  { key: `Dental Hygienist`, value: `dental_hygienist` },
  { key: `Dentist`, value: `dentist` },
  { key: `Dermatologist`, value: `dermatologist` },
  { key: `Design Engineer`, value: `design_engineer` },
  { key: `Designer`, value: `designer` },
  { key: `Desktop Publisher`, value: `desktop_publisher` },
  { key: `Detective`, value: `detective` },
  { key: `DevOps Engineer`, value: `devops_engineer` },
  { key: `Digital Marketer`, value: `digital_marketer` },
  { key: `Digital Marketing Manager`, value: `digital_marketing_manager` },
  { key: `Digital Strategist`, value: `digital_strategist` },
  { key: `Direct Salesperson`, value: `direct_salesperson` },
  { key: `Director`, value: `director` },
  { key: `Director of Communications`, value: `director_of_communications` },
  { key: `Director of Development`, value: `director_of_development` },
  {
    key: `Director of Global Operations`,
    value: `director_of_global_operations`,
  },
  { key: `Director of Inside Sales`, value: `director_of_inside_sales` },
  {
    key: `Director of Learning and Development`,
    value: `director_of_learning_and_development`,
  },
  { key: `Director of Operations`, value: `director_of_operations` },
  { key: `Director of Sales`, value: `director_of_sales` },
  { key: `Dispatcher`, value: `dispatcher` },
  { key: `Distribution Manager`, value: `distribution_manager` },
  { key: `District Manager`, value: `district_manager` },
  { key: `DJ`, value: `dj` },
  { key: `Doctor`, value: `doctor` },
  { key: `Document Control Specialist`, value: `document_control_specialist` },
  { key: `Drafter`, value: `drafter` },
  { key: `Driver`, value: `driver` },
  {
    key: `eCommerce Marketing Specialist`,
    value: `ecommerce_marketing_specialist`,
  },
  { key: `Economist`, value: `economist` },
  { key: `Editor`, value: `editor` },
  { key: `Electrical Engineer`, value: `electrical_engineer` },
  { key: `Electrician`, value: `electrician` },
  { key: `Electronic Technician`, value: `electronic_technician` },
  { key: `Elementary School Teacher`, value: `elementary_school_teacher` },
  {
    key: `Employee Engagement Specialist`,
    value: `employee_engagement_specialist`,
  },
  { key: `Engineer`, value: `engineer` },
  { key: `Engineering Manager`, value: `engineering_manager` },
  { key: `Engineering Technician`, value: `engineering_technician` },
  { key: `English Teacher`, value: `english_teacher` },
  { key: `Entrepreneur`, value: `entrepreneur` },
  { key: `Environmental Engineer`, value: `environmental_engineer` },
  { key: `Environmental Scientist`, value: `environmental_scientist` },
  { key: `Epidemiologist`, value: `epidemiologist` },
  { key: `Equipment Operator`, value: `equipment_operator` },
  { key: `ETL Developer`, value: `etl_developer` },
  { key: `Evangelist`, value: `evangelist` },
  { key: `Event Coordinator`, value: `event_coordinator` },
  { key: `Event Planner`, value: `event_planner` },
  { key: `Executive Assistant`, value: `executive_assistant` },
  { key: `Executive Chef`, value: `executive_chef` },
  { key: `Expert`, value: `expert` },
  { key: `Facilities Manager`, value: `facilities_manager` },
  { key: `Family Physician`, value: `family_physician` },
  { key: `Fashion Designer`, value: `fashion_designer` },
  { key: `Federal Agent`, value: `federal_agent` },
  { key: `Field Service Technician`, value: `field_service_technician` },
  { key: `File Clerk`, value: `file_clerk` },
  { key: `Film Critic`, value: `film_critic` },
  { key: `Film Director`, value: `film_director` },
  { key: `Film Producer`, value: `film_producer` },
  { key: `Finance Director`, value: `finance_director` },
  { key: `Finance Manager`, value: `finance_manager` },
  { key: `Financial Analyst`, value: `financial_analyst` },
  { key: `Financial Manager`, value: `financial_manager` },
  { key: `Financial Planner`, value: `financial_planner` },
  {
    key: `Financial Services Representative`,
    value: `financial_services_representative`,
  },
  { key: `Firefighter`, value: `firefighter` },
  { key: `Fitness Trainer`, value: `fitness_trainer` },
  { key: `Flight Attendant`, value: `flight_attendant` },
  { key: `Food Scientist`, value: `food_scientist` },
  { key: `Food Service Manager`, value: `food_service_manager` },
  { key: `Foreign Language Teacher`, value: `foreign_language_teacher` },
  { key: `Forensic Scientist`, value: `forensic_scientist` },
  { key: `Forklift Operator`, value: `forklift_operator` },
  { key: `Founder`, value: `founder` },
  { key: `Freelance Writer`, value: `freelance_writer` },
  { key: `Front Desk Clerk`, value: `front_desk_clerk` },
  { key: `Front-end Developer`, value: `front-end_developer` },
  { key: `Full-stack Developer`, value: `full-stack_developer` },
  { key: `Fundraiser`, value: `fundraiser` },
  { key: `Furniture Designer`, value: `furniture_designer` },
  { key: `Game Designer`, value: `game_designer` },
  { key: `Game Developer`, value: `game_developer` },
  { key: `General Counsel`, value: `general_counsel` },
  { key: `Generalist`, value: `generalist` },
  { key: `Geological Engineer`, value: `geological_engineer` },
  { key: `Geologist`, value: `geologist` },
  { key: `Ghostwriter`, value: `ghostwriter` },
  { key: `Grant Writer`, value: `grant_writer` },
  { key: `Graphic Designer`, value: `graphic_designer` },
  { key: `Handyman`, value: `handyman` },
  { key: `Hardware Engineer`, value: `hardware_engineer` },
  { key: `Head of Human Resources`, value: `head_of_human_resources` },
  { key: `Head of Sales`, value: `head_of_sales` },
  { key: `Heavy Equipment Operator`, value: `heavy_equipment_operator` },
  { key: `Help Desk`, value: `help_desk` },
  { key: `Help Desk Worker`, value: `help_desk_worker` },
  { key: `Human Resources`, value: `human_resources` },
  { key: `Human Resources Administrator`, value: `hr_administrator` },
  { key: `Human Resources Advisor`, value: `hr_advisor` },
  { key: `Human Resources Analyst`, value: `hr_analyst` },
  { key: `Human Resources Assistant`, value: `hr_assistant` },
  { key: `Human Resources Assistant Manager`, value: `hr_assistant_manager` },
  { key: `Human Resources Associate`, value: `hr_associate` },
  { key: `Human Resources Audit Consultant`, value: `hr_audit_consultant` },
  { key: `Human Resources Audit Coordinator`, value: `hr_audit_coordinator` },
  { key: `Human Resources Audit Director`, value: `hr_audit_director` },
  { key: `Human Resources Audit Manager`, value: `hr_audit_manager` },
  { key: `Human Resources Audit Specialist`, value: `hr_audit_specialist` },
  {
    key: `Human Resources Benefits Administrator`,
    value: `hr_benefits_administrator`,
  },
  { key: `Human Resources Benefits Analyst`, value: `hr_benefits_analyst` },
  {
    key: `Human Resources Benefits Consultant`,
    value: `hr_benefits_consultant`,
  },
  {
    key: `Human Resources Benefits Coordinator`,
    value: `hr_benefits_coordinator`,
  },
  { key: `Human Resources Benefits Manager`, value: `hr_benefits_manager` },
  {
    key: `Human Resources Benefits Specialist`,
    value: `hr_benefits_specialist`,
  },
  { key: `Human Resources Business Analyst`, value: `hr_business_analyst` },
  { key: `Human Resources Business Partner`, value: `hr_business_partner` },
  {
    key: `Human Resources Business Partner Consultant`,
    value: `hr_business_partner_consultant`,
  },
  {
    key: `Human Resources Business Partner Coordinator`,
    value: `hr_business_partner_coordinator`,
  },
  {
    key: `Human Resources Business Partner Director`,
    value: `hr_business_partner_director`,
  },
  {
    key: `Human Resources Business Partner Manager`,
    value: `hr_business_partner_manager`,
  },
  {
    key: `Human Resources Business Partner Specialist`,
    value: `hr_business_partner_specialist`,
  },
  {
    key: `Human Resources Communications Consultant`,
    value: `hr_communications_consultant`,
  },
  {
    key: `Human Resources Communications Coordinator`,
    value: `hr_communications_coordinator`,
  },
  {
    key: `Human Resources Communications Director`,
    value: `hr_communications_director`,
  },
  {
    key: `Human Resources Communications Manager`,
    value: `hr_communications_manager`,
  },
  {
    key: `Human Resources Communications Specialist`,
    value: `hr_communications_specialist`,
  },
  { key: `Human Resources Compliance Analyst`, value: `hr_compliance_analyst` },
  {
    key: `Human Resources Compliance Consultant`,
    value: `hr_compliance_consultant`,
  },
  {
    key: `Human Resources Compliance Coordinator`,
    value: `hr_compliance_coordinator`,
  },
  { key: `Human Resources Compliance Manager`, value: `hr_compliance_manager` },
  {
    key: `Human Resources Compliance Specialist`,
    value: `hr_compliance_specialist`,
  },
  { key: `Human Resources Consultant`, value: `hr_consultant` },
  { key: `Human Resources Coordinator`, value: `hr_coordinator` },
  { key: `Human Resources Data Analyst`, value: `hr_data_analyst` },
  { key: `Human Resources Data Coordinator`, value: `hr_data_coordinator` },
  { key: `Human Resources Director`, value: `hr_director` },
  { key: `Human Resources Executive`, value: `hr_executive` },
  { key: `Human Resources Generalist`, value: `hr_generalist` },
  {
    key: `Human Resources Information Analyst`,
    value: `hr_information_analyst`,
  },
  {
    key: `Human Resources Information Systems Administrator`,
    value: `hr_information_systems_administrator`,
  },
  {
    key: `Human Resources Information Systems Analyst`,
    value: `hr_information_systems_analyst`,
  },
  {
    key: `Human Resources Information Systems Business Analyst`,
    value: `hr_information_systems_business_analyst`,
  },
  {
    key: `Human Resources Information Systems Consultant`,
    value: `hr_information_systems_consultant`,
  },
  {
    key: `Human Resources Information Systems Coordinator`,
    value: `hr_information_systems_coordinator`,
  },
  {
    key: `Human Resources Information Systems Data Analyst`,
    value: `hr_information_systems_data_analyst`,
  },
  {
    key: `Human Resources Information Systems Director`,
    value: `hr_information_systems_director`,
  },
  {
    key: `Human Resources Information Systems Manager`,
    value: `hr_information_systems_manager`,
  },
  {
    key: `Human Resources Information Systems Project Manager`,
    value: `hr_information_systems_project_manager`,
  },
  {
    key: `Human Resources Information Systems Specialist`,
    value: `hr_information_systems_specialist`,
  },
  {
    key: `Human Resources Information Systems Support Specialist`,
    value: `hr_information_systems_support_specialist`,
  },
  {
    key: `Human Resources Information Systems Trainer`,
    value: `hr_information_systems_trainer`,
  },
  { key: `Human Resources Manager`, value: `hr_manager` },
  { key: `Human Resources Metrics Analyst`, value: `hr_metrics_analyst` },
  { key: `Human Resources Officer`, value: `hr_officer` },
  { key: `Human Resources Operations Analyst`, value: `hr_operations_analyst` },
  {
    key: `Human Resources Operations Consultant`,
    value: `hr_operations_consultant`,
  },
  {
    key: `Human Resources Operations Coordinator`,
    value: `hr_operations_coordinator`,
  },
  {
    key: `Human Resources Operations Director`,
    value: `hr_operations_director`,
  },
  { key: `Human Resources Operations Manager`, value: `hr_operations_manager` },
  { key: `Human Resources Recruiter`, value: `hr_recruiter` },
  { key: `Human Resources Reporting Analyst`, value: `hr_reporting_analyst` },
  { key: `Human Resources Representative`, value: `hr_representative` },
  {
    key: `Human Resources Service Center Analyst`,
    value: `hr_service_center_analyst`,
  },
  {
    key: `Human Resources Service Center Consultant`,
    value: `hr_service_center_consultant`,
  },
  {
    key: `Human Resources Service Center Coordinator`,
    value: `hr_service_center_coordinator`,
  },
  {
    key: `Human Resources Service Center Manager`,
    value: `hr_service_center_manager`,
  },
  {
    key: `Human Resources Service Center Specialist`,
    value: `hr_service_center_specialist`,
  },
  { key: `Human Resources Specialist`, value: `hr_specialist` },
  { key: `Human Resources Support`, value: `hr_support` },
  {
    key: `Human Resources Systems, Processes, and Benefits Specialist`,
    value: `hr_systems_processes_and_benefits_specialist`,
  },
  {
    key: `Information Security Analyst`,
    value: `information_security_analyst`,
  },
  { key: `Instructor`, value: `instructor` },
  { key: `Interaction Designer`, value: `interaction_designer` },
  { key: `Intern`, value: `intern` },
  { key: `Interpreter`, value: `interpreter` },
  { key: `Iron Worker`, value: `iron_worker` },
  { key: `IT Manager`, value: `it_manager` },
  { key: `IT Professional`, value: `it_professional` },
  { key: `Journalist`, value: `journalist` },
  {
    key: `Learning and Development Specialist`,
    value: `learning_and_development_specialist`,
  },
  { key: `Librarian`, value: `librarian` },
  { key: `Machine Learning Analyst`, value: `ml_analyst` },
  { key: `Machine Learning Engineer`, value: `ml_engineer` },
  { key: `Machine Learning Lead`, value: `ml_lead` },
  { key: `Machine Learning Manager`, value: `ml_manager` },
  { key: `Machine Learning Scientist`, value: `ml_scientist` },
  { key: `Machine Learning Specialist`, value: `ml_specialist` },
  { key: `Maintenance Engineer`, value: `maintenance_engineer` },
  { key: `Manager`, value: `manager` },
  { key: `Managing Director`, value: `managing_director` },
  { key: `Managing Member`, value: `managing_member` },
  { key: `Managing Partner`, value: `managing_partner` },
  { key: `Market Development Manager`, value: `market_development_manager` },
  { key: `Market Researcher`, value: `market_researcher` },
  {
    key: `Marketing Communications Manager`,
    value: `marketing_communications_manager`,
  },
  { key: `Marketing Consultant`, value: `marketing_consultant` },
  { key: `Marketing Director`, value: `marketing_director` },
  { key: `Marketing Insights Analyst`, value: `marketing_insights_analyst` },
  { key: `Marketing Manager`, value: `marketing_manager` },
  { key: `Marketing Research Analyst`, value: `marketing_research_analyst` },
  { key: `Marketing Specialist`, value: `marketing_specialist` },
  { key: `Mason`, value: `mason` },
  { key: `Massage Therapy`, value: `massage_therapy` },
  {
    key: `Master Data Management (MDM) Analyst`,
    value: `master_data_management_(mdm)_analyst`,
  },
  { key: `Master Data Management Lead`, value: `master_data_management_lead` },
  {
    key: `Master Data Management Specialist`,
    value: `master_data_management_specialist`,
  },
  { key: `MDM Architect`, value: `mdm_architect` },
  { key: `MDM Developer`, value: `mdm_developer` },
  { key: `MDM Manager`, value: `mdm_manager` },
  { key: `Mechanical Engineer`, value: `mechanical_engineer` },
  { key: `Media Buyer`, value: `media_buyer` },
  { key: `Media Producer`, value: `media_producer` },
  { key: `Media Relations Coordinator`, value: `media_relations_coordinator` },
  { key: `Medical Administrator`, value: `medical_administrator` },
  { key: `Medical Laboratory Tech`, value: `medical_laboratory_tech` },
  { key: `Medical Researcher`, value: `medical_researcher` },
  { key: `Medical Transcriptionist`, value: `medical_transcriptionist` },
  { key: `Merchandising Associate`, value: `merchandising_associate` },
  { key: `Mining Engineer`, value: `mining_engineer` },
  { key: `Mobile Developer`, value: `mobile_developer` },
  { key: `Molecular Scientist`, value: `molecular_scientist` },
  { key: `Musician`, value: `musician` },
  { key: `Network Administrator`, value: `network_administrator` },
  { key: `Network Engineer`, value: `network_engineer` },
  { key: `Nuclear Engineer`, value: `nuclear_engineer` },
  { key: `Nurse`, value: `nurse` },
  { key: `Nurse Practitioner`, value: `nurse_practitioner` },
  { key: `Office Assistant`, value: `office_assistant` },
  { key: `Office Clerk`, value: `office_clerk` },
  { key: `Office Manager`, value: `office_manager` },
  { key: `Operations Analyst`, value: `operations_analyst` },
  { key: `Operations Assistant`, value: `operations_assistant` },
  { key: `Operations Coordinator`, value: `operations_coordinator` },
  { key: `Operations Director`, value: `operations_director` },
  { key: `Operations Manager`, value: `operations_manager` },
  { key: `Operations Professional`, value: `operations_professional` },
  { key: `Operator`, value: `operator` },
  { key: `Orderly`, value: `orderly` },
  { key: `Outside Sales Manager`, value: `outside_sales_manager` },
  { key: `Owner`, value: `owner` },
  { key: `Painter`, value: `painter` },
  { key: `Payroll Clerk`, value: `payroll_clerk` },
  { key: `Payroll Manager`, value: `payroll_manager` },
  { key: `People Analyst`, value: `people_analyst` },
  { key: `Personal Trainer`, value: `personal_trainer` },
  { key: `Petroleum Engineer`, value: `petroleum_engineer` },
  { key: `Pharmacist`, value: `pharmacist` },
  { key: `Pharmacy Assistant`, value: `pharmacy_assistant` },
  { key: `Phlebotomist`, value: `phlebotomist` },
  { key: `Physical Therapist`, value: `physical_therapist` },
  { key: `Physical Therapy Assistant`, value: `physical_therapy_assistant` },
  { key: `Physicist`, value: `physicist` },
  { key: `Pipefitter`, value: `pipefitter` },
  { key: `Plant Engineer`, value: `plant_engineer` },
  { key: `Plumber`, value: `plumber` },
  { key: `Political Scientist`, value: `political_scientist` },
  {
    key: `Predictive Analytics Developer`,
    value: `predictive_analytics_developer`,
  },
  { key: `Predictive Analytics Lead`, value: `predictive_analytics_lead` },
  {
    key: `Predictive Analytics Manager`,
    value: `predictive_analytics_manager`,
  },
  {
    key: `Predictive Analytics Modeler`,
    value: `predictive_analytics_modeler`,
  },
  {
    key: `Predictive Analytics Specialist`,
    value: `predictive_analytics_specialist`,
  },
  { key: `President`, value: `president` },
  { key: `Principal`, value: `principal` },
  { key: `Producer`, value: `producer` },
  { key: `Product Manager`, value: `product_manager` },
  { key: `Product Owner`, value: `product_owner` },
  { key: `Production Engineer`, value: `production_engineer` },
  { key: `Professor`, value: `professor` },
  { key: `Program Administrator`, value: `program_administrator` },
  { key: `Program Manager`, value: `program_manager` },
  { key: `Project Manager`, value: `project_manager` },
  { key: `Proofreader`, value: `proofreader` },
  { key: `Proposal Writer`, value: `proposal_writer` },
  { key: `Proprietor`, value: `proprietor` },
  { key: `Public Relations`, value: `public_relations` },
  { key: `Public Relations Specialist`, value: `public_relations_specialist` },
  {
    key: `Python and Data Science Corporate Trainer and Consultant`,
    value: `python_and_data_science_corporate_trainer_and_consultant`,
  },
  { key: `QA Engineer`, value: `qa_engineer` },
  { key: `Quality Control Coordinator`, value: `quality_control_coordinator` },
  { key: `Quality Engineer`, value: `quality_engineer` },
  { key: `Quantitative Analyst`, value: `quantitative_analyst` },
  { key: `Real Estate Broker`, value: `real_estate_broker` },
  { key: `Receptionist`, value: `receptionist` },
  { key: `Records Clerk`, value: `records_clerk` },
  { key: `Recruiter`, value: `recruiter` },
  { key: `Research Assistant`, value: `research_assistant` },
  { key: `Research Manager`, value: `research_manager` },
  { key: `Research Scientist`, value: `research_scientist` },
  { key: `Researcher`, value: `researcher` },
  { key: `Retail Worker`, value: `retail_worker` },
  { key: `Risk Manager`, value: `risk_manager` },
  { key: `Roofer`, value: `roofer` },
  { key: `Safety Engineer`, value: `safety_engineer` },
  { key: `Sales Analyst`, value: `sales_analyst` },
  { key: `Sales Associate`, value: `sales_associate` },
  { key: `Sales Engineer`, value: `sales_engineer` },
  { key: `Sales Manager`, value: `sales_manager` },
  { key: `Sales Representative`, value: `sales_representative` },
  { key: `Sales Specialist`, value: `sales_specialist` },
  { key: `Scientist`, value: `scientist` },
  { key: `Screenwriter`, value: `screenwriter` },
  { key: `Scrum Master`, value: `scrum_master` },
  { key: `Secretary`, value: `secretary` },
  { key: `Security Engineer`, value: `security_engineer` },
  { key: `Senior Data Scientist`, value: `senior_data_scientist` },
  { key: `Senior Developer Advocate`, value: `senior_developer_advocate` },
  { key: `SEO Manager`, value: `seo_manager` },
  { key: `Sheet Metal Worker`, value: `sheet_metal_worker` },
  { key: `Social Media Assistant`, value: `social_media_assistant` },
  { key: `Social Media Manager`, value: `social_media_manager` },
  { key: `Social Media Specialist`, value: `social_media_specialist` },
  { key: `Sociologist`, value: `sociologist` },
  { key: `Software Architect`, value: `software_architect` },
  { key: `Software Developer`, value: `software_developer` },
  { key: `Software Developer Advocate`, value: `software_developer_advocate` },
  { key: `Software Engineer`, value: `software_engineer` },
  {
    key: `Solar Photovoltaic Installer`,
    value: `solar_photovoltaic_installer`,
  },
  { key: `Solutions Architect`, value: `solutions_architect` },
  { key: `Specialist`, value: `specialist` },
  { key: `Speechwriter`, value: `speechwriter` },
  { key: `SQL Developer`, value: `sql_developer` },
  { key: `Staff`, value: `staff` },
  { key: `Statistician`, value: `statistician` },
  { key: `Store Manager`, value: `store_manager` },
  { key: `Strategist`, value: `strategist` },
  { key: `Superintendent`, value: `superintendent` },
  { key: `Supervisor`, value: `supervisor` },
  { key: `Supply Manager`, value: `supply_manager` },
  { key: `Support Specialist`, value: `support_specialist` },
  { key: `System Administrator`, value: `system_administrator` },
  { key: `Systems Administrator`, value: `systems_administrator` },
  { key: `Systems Developer`, value: `systems_developer` },
  { key: `Talent Acquisition Partner`, value: `talent_acquisition_partner` },
  {
    key: `Talent Acquisition Specialist`,
    value: `talent_acquisition_specialist`,
  },
  { key: `Talent Acquisition Manager`, value: `talent_acquisition_manager` },
  {
    key: `Talent Acquisition Recruiter`,
    value: `talent_acquisition_recruiter`,
  },
  { key: `Taper`, value: `taper` },
  { key: `Teacher`, value: `teacher` },
  { key: `Teaching Assistant`, value: `teaching_assistant` },
  { key: `Tech Lead`, value: `tech_lead` },
  { key: `Technical Recruiter`, value: `technical_recruiter` },
  { key: `Technical Specialist`, value: `technical_specialist` },
  {
    key: `Technical Support Specialist`,
    value: `technical_support_specialist`,
  },
  { key: `Technical Writer`, value: `technical_writer` },
  { key: `Technician`, value: `technician` },
  { key: `Test Engineer`, value: `test_engineer` },
  { key: `Title Analyst`, value: `title_analyst` },
  { key: `Title Researcher`, value: `title_researcher` },
  { key: `Trainee`, value: `trainee` },
  { key: `Trainer`, value: `trainer` },
  { key: `Translator`, value: `translator` },
  { key: `Travel Nurse`, value: `travel_nurse` },
  { key: `Travel Writer`, value: `travel_writer` },
  { key: `Tutor`, value: `tutor` },
  { key: `UX Designer`, value: `ux_designer` },
  {
    key: `UX Designer and UI Developer`,
    value: `ux_designer_and_ui_developer`,
  },
  { key: `UX/UI Designer`, value: `ux_ui_designer` },
  {
    key: `Vehicle or Equipment Cleaner`,
    value: `vehicle_or_equipment_cleaner`,
  },
  { key: `Vice President of Human Resources`, value: `vp_human_resources` },
  { key: `Vice President of Marketing`, value: `vp_marketing` },
  { key: `Vice President of Operations`, value: `vp_operations` },
  {
    key: `Vice President of Talent Acquisition`,
    value: `vp_talent_acquisition`,
  },
  { key: `Vice President of Finance`, value: `vp_finance` },
  { key: `Vice President of Sales`, value: `vp_sales` },
  { key: `Video Game Writer`, value: `video_game_writer` },
  { key: `Videographer`, value: `videographer` },
  { key: `Virtual Assistant`, value: `virtual_assistant` },
  { key: `Waiter`, value: `waiter` },
  { key: `Waitress`, value: `waitress` },
  { key: `Web Designer`, value: `web_designer` },
  { key: `Web Developer`, value: `web_developer` },
  { key: `Web Producer`, value: `web_producer` },
  { key: `Welder`, value: `welder` },
  { key: `Welfare Manager`, value: `welfare_manager` },
  { key: `Well Driller`, value: `well_driller` },
  { key: `Worker`, value: `worker` },
  { key: `Writer`, value: `writer` },
];

const input_obj_arr = [{ key: "_User Input", value: "_user_input" }];

input_obj_arr.push(job_titles_obj_arr);

job_titles_obj_arr = input_obj_arr.flat();

async function job_title(tp) {
  let job_title_obj;

  const job_title_suggester = await tp.system.suggester(
    (item) => item.key,
    job_titles_obj_arr,
    false,
    "Job Title?"
  );

  if (job_title_suggester.value == "_user_input") {
    const job_title_input = await tp.system.prompt("Job Title?");
    const job_title_input_fmt = job_title_input
      .replaceAll(/,/g, "")
      .replaceAll(/\s/g, "_")
      .replaceAll(/\//g, "-")
      .replaceAll(/&/g, "and")
      .toLowerCase();

    job_title_obj = {
      key: job_title_input,
      value: job_title_input_fmt,
    };
  } else {
    job_title_obj = {
      key: job_title_suggester.key,
      value: job_title_suggester.value,
    };
  }

  return job_title_obj;
}

module.exports = job_title;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET JOB TITLE
//---------------------------------------------------------
const job_title_obj = await tp.user.suggester_job_title(tp);
const job_title_name = job_title_obj.key;
const job_title_value = job_title_obj.value;	  
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[51_41_parent_job_application]]
2. [[61_contact]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[job_title.js]]

### Outgoing Snippet Links

<!-- Link related snippets here  -->

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
