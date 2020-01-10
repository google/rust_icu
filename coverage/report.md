# Implementation coverage report

| Header | Implemented |
| ------ | ----------- |
| `ustring.h` | 3 / 61 | 
| `ucal.h` | 5 / 45 | 
| `udat.h` | 6 / 38 | 
| `udata.h` | 2 / 8 | 
| `uenum.h` | 7 / 8 | 
| `uloc.h` | 9 / 41 | 
| `ustring.h` | 3 / 61 | 
| `utext.h` | 3 / 28 | 
# Unimplemented functions per header


# Header: `ustring.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `UChar*` |
| | `u_strFromUTF8` |
| | `u_strToUTF8` |
| `u_austrcpy` | |
| `u_austrncpy` | |
| `u_countChar32` | |
| `u_memcasecmp` | |
| `u_memchr` | |
| `u_memchr32` | |
| `u_memcmp` | |
| `u_memcmpCodePointOrder` | |
| `u_memcpy` | |
| `u_memmove` | |
| `u_memrchr` | |
| `u_memrchr32` | |
| `u_memset` | |
| `u_strcasecmp` | |
| `u_strCaseCompare` | |
| `u_strcat` | |
| `u_strchr` | |
| `u_strchr32` | |
| `u_strcmp` | |
| `u_strcmpCodePointOrder` | |
| `u_strCompare` | |
| `u_strCompareIter` | |
| `u_strcpy` | |
| `u_strcspn` | |
| `u_strFindFirst` | |
| `u_strFindLast` | |
| `u_strFoldCase` | |
| `u_strFromJavaModifiedUTF8WithSub` | |
| `u_strFromUTF32` | |
| `u_strFromUTF32WithSub` | |
| `u_strFromUTF8Lenient` | |
| `u_strFromUTF8WithSub` | |
| `u_strFromWCS` | |
| `u_strHasMoreChar32Than` | |
| `u_strlen` | |
| `u_strncasecmp` | |
| `u_strncat` | |
| `u_strncmp` | |
| `u_strncmpCodePointOrder` | |
| `u_strncpy` | |
| `u_strpbrk` | |
| `u_strrchr` | |
| `u_strrchr32` | |
| `u_strrstr` | |
| `u_strspn` | |
| `u_strstr` | |
| `u_strToJavaModifiedUTF8` | |
| `u_strtok_r` | |
| `u_strToLower` | |
| `u_strToTitle` | |
| `u_strToUpper` | |
| `u_strToUTF32` | |
| `u_strToUTF32WithSub` | |
| `u_strToUTF8WithSub` | |
| `u_strToWCS` | |
| `u_uastrcpy` | |
| `u_uastrncpy` | |
| `u_unescape` | |
| `u_unescapeAt` | |

# Header: `ucal.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `ucal_close` |
| | `ucal_getDefaultTimeZone` |
| | `ucal_getNow` |
| | `ucal_open` |
| | `ucal_setDefaultTimeZone` |
| `ucal_add` | |
| `ucal_clear` | |
| `ucal_clearField` | |
| `ucal_clone` | |
| `ucal_countAvailable` | |
| `ucal_equivalentTo` | |
| `ucal_get` | |
| `ucal_getAttribute` | |
| `ucal_getAvailable` | |
| `ucal_getCanonicalTimeZoneID` | |
| `ucal_getDayOfWeekType` | |
| `ucal_getDSTSavings` | |
| `ucal_getFieldDifference` | |
| `ucal_getGregorianChange` | |
| `ucal_getKeywordValuesForLocale` | |
| `ucal_getLimit` | |
| `ucal_getLocaleByType` | |
| `ucal_getMillis` | |
| `ucal_getTimeZoneDisplayName` | |
| `ucal_getTimeZoneID` | |
| `ucal_getTimeZoneIDForWindowsID` | |
| `ucal_getTimeZoneTransitionDate` | |
| `ucal_getType` | |
| `ucal_getTZDataVersion` | |
| `ucal_getWeekendTransition` | |
| `ucal_getWindowsTimeZoneID` | |
| `ucal_inDaylightTime` | |
| `ucal_isSet` | |
| `ucal_isWeekend` | |
| `ucal_openCountryTimeZones` | |
| `ucal_openTimeZoneIDEnumeration` | |
| `ucal_openTimeZones` | |
| `ucal_roll` | |
| `ucal_set` | |
| `ucal_setAttribute` | |
| `ucal_setDate` | |
| `ucal_setDateTime` | |
| `ucal_setGregorianChange` | |
| `ucal_setMillis` | |
| `ucal_setTimeZone` | |

# Header: `udat.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `udat_close` |
| | `UDateFormat` |
| | `udat_format` |
| | `udat_open` |
| | `udat_parse` |
| | `udat_setCalendar` |
| `udat_adoptNumberFormat` | |
| `udat_adoptNumberFormatForFields` | |
| `udat_applyPattern` | |
| `udat_applyPatternRelative` | |
| `udat_clone` | |
| `udat_countAvailable` | |
| `udat_countSymbols` | |
| `udat_formatCalendar` | |
| `udat_formatCalendarForFields` | |
| `udat_formatForFields` | |
| `udat_get2DigitYearStart` | |
| `udat_getAvailable` | |
| `udat_getBooleanAttribute` | |
| `udat_getCalendar` | |
| `udat_getContext` | |
| `udat_getLocaleByType` | |
| `udat_getNumberFormat` | |
| `udat_getNumberFormatForField` | |
| `udat_getSymbols` | |
| `udat_isLenient` | |
| `udat_parseCalendar` | |
| `udat_registerOpener` | |
| `udat_set2DigitYearStart` | |
| `udat_setBooleanAttribute` | |
| `udat_setContext` | |
| `udat_setLenient` | |
| `udat_setNumberFormat` | |
| `udat_setSymbols` | |
| `udat_toCalendarDateField` | |
| `udat_toPattern` | |
| `udat_toPatternRelativeDate` | |
| `udat_toPatternRelativeTime` | |
| `udat_unregisterOpener` | |

# Header: `udata.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `UDataMemory` |
| | `udata_setCommonData` |
| `udata_close` | |
| `udata_getInfo` | |
| `udata_getMemory` | |
| `udata_open` | |
| `udata_openChoice` | |
| `udata_setAppData` | |
| `udata_setFileAccess` | |

# Header: `uenum.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `ucal_openCountryTimeZones` |
| | `ucal_openTimeZoneIDENumeration` |
| | `ucal_openTimeZones` |
| | `uenum_close` |
| | `UEnumeration` |
| | `uenum_next` |
| | `uenum_openCharStringsEnumeration` |
| `uenum_count` | |
| `uenum_openFromStringEnumeration` | |
| `uenum_openUCharStringsEnumeration` | |
| `uenum_reset` | |
| `uenum_unext` | |

# Header: `uloc.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `uloc_addLikelySubtags` |
| | `uloc_canonicalize` |
| | `uloc_getCountry` |
| | `uloc_getDefault` |
| | `uloc_getLanguage` |
| | `uloc_getScript` |
| | `uloc_getVariant` |
| | `uloc_minimizeSubtags` |
| | `uloc_setDefault` |
| `uloc_acceptLanguage` | |
| `uloc_acceptLanguageFromHTTP` | |
| `uloc_countAvailable` | |
| `uloc_forLanguageTag` | |
| `uloc_getAvailable` | |
| `uloc_getBaseName` | |
| `uloc_getCharacterOrientation` | |
| `uloc_getDisplayCountry` | |
| `uloc_getDisplayKeyword` | |
| `uloc_getDisplayKeywordValue` | |
| `uloc_getDisplayLanguage` | |
| `uloc_getDisplayName` | |
| `uloc_getDisplayScript` | |
| `uloc_getDisplayVariant` | |
| `uloc_getISO3Country` | |
| `uloc_getISO3Language` | |
| `uloc_getISOCountries` | |
| `uloc_getISOLanguages` | |
| `uloc_getKeywordValue` | |
| `uloc_getLCID` | |
| `uloc_getLineOrientation` | |
| `uloc_getLocaleForLCID` | |
| `uloc_getName` | |
| `uloc_getParent` | |
| `uloc_isRightToLeft` | |
| `uloc_openKeywords` | |
| `uloc_setKeywordValue` | |
| `uloc_toLanguageTag` | |
| `uloc_toLegacyKey` | |
| `uloc_toLegacyType` | |
| `uloc_toUnicodeLocaleKey` | |
| `uloc_toUnicodeLocaleType` | |

# Header: `ustring.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `UChar*` |
| | `u_strFromUTF8` |
| | `u_strToUTF8` |
| `u_austrcpy` | |
| `u_austrncpy` | |
| `u_countChar32` | |
| `u_memcasecmp` | |
| `u_memchr` | |
| `u_memchr32` | |
| `u_memcmp` | |
| `u_memcmpCodePointOrder` | |
| `u_memcpy` | |
| `u_memmove` | |
| `u_memrchr` | |
| `u_memrchr32` | |
| `u_memset` | |
| `u_strcasecmp` | |
| `u_strCaseCompare` | |
| `u_strcat` | |
| `u_strchr` | |
| `u_strchr32` | |
| `u_strcmp` | |
| `u_strcmpCodePointOrder` | |
| `u_strCompare` | |
| `u_strCompareIter` | |
| `u_strcpy` | |
| `u_strcspn` | |
| `u_strFindFirst` | |
| `u_strFindLast` | |
| `u_strFoldCase` | |
| `u_strFromJavaModifiedUTF8WithSub` | |
| `u_strFromUTF32` | |
| `u_strFromUTF32WithSub` | |
| `u_strFromUTF8Lenient` | |
| `u_strFromUTF8WithSub` | |
| `u_strFromWCS` | |
| `u_strHasMoreChar32Than` | |
| `u_strlen` | |
| `u_strncasecmp` | |
| `u_strncat` | |
| `u_strncmp` | |
| `u_strncmpCodePointOrder` | |
| `u_strncpy` | |
| `u_strpbrk` | |
| `u_strrchr` | |
| `u_strrchr32` | |
| `u_strrstr` | |
| `u_strspn` | |
| `u_strstr` | |
| `u_strToJavaModifiedUTF8` | |
| `u_strtok_r` | |
| `u_strToLower` | |
| `u_strToTitle` | |
| `u_strToUpper` | |
| `u_strToUTF32` | |
| `u_strToUTF32WithSub` | |
| `u_strToUTF8WithSub` | |
| `u_strToWCS` | |
| `u_uastrcpy` | |
| `u_uastrncpy` | |
| `u_unescape` | |
| `u_unescapeAt` | |

# Header: `utext.h`

| Unimplemented | Implemented |
| ------------- | ----------- |
| | `utext_clone` |
| | `utext_close` |
| | `utext_open` |
| `utext_char32At` | |
| `utext_copy` | |
| `utext_current32` | |
| `utext_equals` | |
| `utext_extract` | |
| `utext_freeze` | |
| `utext_getNativeIndex` | |
| `utext_getPreviousNativeIndex` | |
| `utext_hasMetaData` | |
| `utext_isLengthExpensive` | |
| `utext_isWritable` | |
| `utext_moveIndex32` | |
| `utext_nativeLength` | |
| `utext_next32` | |
| `utext_next32From` | |
| `utext_openCharacterIterator` | |
| `utext_openConstUnicodeString` | |
| `utext_openReplaceable` | |
| `utext_openUChars` | |
| `utext_openUnicodeString` | |
| `utext_openUTF8` | |
| `utext_previous32` | |
| `utext_previous32From` | |
| `utext_replace` | |
| `utext_setNativeIndex` | |
| `utext_setup` | |
