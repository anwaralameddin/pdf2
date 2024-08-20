/// This module aims to provide a unified interface to process all PDF
/// dictionary types.
/// TODO However, it remains unimplemented.

// TODO Remove this attribute when the list is complete and From<&str> is
// implemented
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum DictionaryType {
    /// REFERENCE: [Table 16 — Additional entries specific to an object stream dictionary]
    ObjStm,
    /// REFERENCE: [Table 17 — Additional entries specific to a cross-reference stream dictionary]
    XRef,
    /// REFERENCE: [Table 20 — Entries common to all encryption dictionaries]
    Encrypt,
    /// REFERENCE:
    /// - [Table 25 — Entries common to all crypt filter dictionaries]
    /// - [Table 27 — Additional crypt filter dictionary entries for public-key security handlers]
    CryptFilter,
    /// REFERENCE: [Table 28 — Entries in an encrypted payload dictionary]
    EncryptedPayload,
    /// REFERENCE:
    /// - [Table 29 — Entries in the catalog dictionary]
    /// - [Table 245 — Entries in the FDF catalog dictionary]
    Catalog,
    /// REFERENCE: [Table 30 — Required entries in a page tree node]
    Pages,
    /// REFERENCE:
    /// - [Table 31 — Entries in a page object]
    /// - [Table 363 — Property list entries for artifacts]
    Page,
    /// REFERENCE: [Table 31 — Entries in a page object]
    Template,
    /// REFERENCE: [Table 43 — Entries in a file specification dictionary]
    Filespec,
    /// REFERENCE: [Table 44 — Additional entries in an embedded file stream dictionary]
    EmbeddedFile,
    /// REFERENCE: [Table 47 — Entries in a collection subitem dictionary]
    CollectionSubitem,
    /// REFERENCE: [Table 48 — Entries in an extensions dictionary]
    Extensions,
    /// REFERENCE: [Table 49 — Entries in a developer extensions dictionary]
    DeveloperExtensions,
    /// REFERENCE:
    /// - [Table 62 — Entries in a CalGray colour space dictionary]
    /// - [Table 63 — Entries in a CalRGB colour space dictionary]
    /// - [Table 64 — Entries in a Lab colour space dictionary]
    /// - [Table 70 — Entries in a DeviceN colour space attributes dictionary]
    ColorSpace,
    /// REFERENCE:
    /// - [Table 74 — Additional entries specific to a Type 1 pattern dictionary]
    /// - [Table 75 — Entries in a Type 2 pattern dictionary]
    Pattern,
    /// REFERENCE: [93 — Additional entries specific to a Type 1 form dictionary]
    XObject,
    /// REFERENCE:
    /// - [Table 94 — Entries common to all group attributes dictionaries]
    /// - [Table 145 — Additional entries specific to a transparency group attributes dictionary]
    Group,
    /// REFERENCE:
    /// - [Table 96 — Entries in an optional content group dictionary]
    /// - [Table 98 — Entries in the optional content properties dictionary]
    OCG,
    /// REFERENCE: [Table 97 — Entries in an optional content membership dictionary]
    OCMD,
    /// REFERENCE: [Table 112 — Entries in an encoding dictionary]
    Encoding,
    /// REFERENCE:
    /// - [Table 114 — Entries in a CIDSystemInfo dictionary]
    /// - [Table 115 — Entries in a CIDFont dictionary]
    /// - [Table 119 — Entries in a Type 0 font dictionary]
    /// - [Table 120 — Entries common to all font descriptors]
    Font,
    /// REFERENCE: [Table 118 — Additional entries in a CMap stream dictionary]
    CMap,
    /// REFERENCE: [Table 122 — Additional font descriptor entries for CIDFonts]
    FontDescriptor,
    /// REFERENCE:
    /// - [Table 125 — Additional entries in an embedded font stream dictionary]
    /// - [Table 128 — Entries in a Type 1 halftone dictionary]
    /// - [Table 129 — Additional entries specific to a Type 6 halftone dictionary]
    /// - [Table 130 — Additional entries specific to a Type 10 halftone dictionary]
    /// - [Table 131 — Additional entries specific to a Type 16 halftone dictionary]
    /// - [Table 132 — Entries in a Type 5 halftone dictionary]
    Halftone,
    /// REFERENCE:
    /// - [Table 142 — Entries in a soft-mask dictionary]
    /// - [Table 143 — Restrictions on the entries in a soft-mask image dictionary]
    Mask,
    /// REFERENCE: [Table 150 — Entries in the outline dictionary]
    Outlines,
    /// REFERENCE: [Table 153 — Entries in a collection dictionary]
    Collection,
    /// REFERENCE: [Table 154 — Entries in a collection schema dictionary]
    CollectionSchema,
    /// REFERENCE: [Table 155 — Entries in a collection field dictionary]
    CollectionField,
    /// REFERENCE: [Table 156 — Entries in a collection sort dictionary]
    CollectionSort,
    /// REFERENCE: [Table 157 — Entries in a collection colors dictionary]
    CollectionColors, // PDF 2.0
    /// REFERENCE: [Table 158 — Entries in a collection split dictionary]
    CollectionSplit, // PDF 2.0
    /// REFERENCE:
    /// - [Table 160 — Entries in a navigator dictionary]
    /// - [Table 165 — Entries in a navigation node dictionary]
    Navigator, // PDF 2.0
    /// REFERENCE: [Table 162 — Entries in a thread dictionary]
    Thread,
    /// REFERENCE: [Table 163 — Entries in a bead dictionary]
    Bead,
    /// REFERENCE: [Table 164 — Entries in a transition dictionary]
    Trans,
    /// REFERENCE: [Table 165 — Entries in a navigation node dictionary]
    NavNode,
    /// REFERENCE:
    /// - [Table 166 — Entries common to all annotation dictionaries]
    /// - [Table 172 — Additional entries in an annotation dictionary specific to markup annotations]
    /// - [Table 173 — Additional entries in markup annotation dictionaries specific to external data]
    /// - [Table 175 — Additional entries specific to a text annotation dictionary]
    /// - [Table 176 — Additional entries specific to a link annotation dictionary]
    /// - [Table 177 — Additional entries specific to a free text annotation dictionary]
    /// - [Table 178 — Additional entries specific to a line annotation dictionary]
    /// - [Table 180 — Additional entries specific to a square or circle annotation dictionary]
    /// - [Table 181 — Additional entries specific to a polygon or polyline annotation dictionary]
    /// - [Table 182 — Additional entries specific to text markup annotations]
    /// - [Table 183 — Additional entries specific to a caret annotation dictionary]
    /// - [Table 184 — Additional entries specific to a rubber stamp annotation dictionary]
    /// - [Table 185 — Additional entries specific to an ink annotation dictionary]
    /// - [Table 186 — Additional entries specific to a popup annotation dictionary]
    /// - [Table 187 — Additional entries specific to a file attachment annotation dictionary]
    /// - [Table 188 — Additional entries specific to a sound annotation dictionary]
    /// - [Table 189 — Additional entries specific to a movie annotation dictionary]
    /// - [Table 190 — Additional entries specific to a screen annotation dictionary]
    /// - [Table 191 — Additional entries specific to a widget annotation]
    /// - [Table 192 — Entries in an appearance characteristics dictionary]
    /// - [Table 193 — Additional entries specific to a watermark annotation dictionary]
    /// - [Table 195 — Additional entries specific to a redaction annotation dictionary]
    /// - [Table 197 — Entries in an annotation’s additional-actions dictionary]
    /// - [Table 333 — Additional entries specific to a RichMedia annotation dictionary]
    /// - [Table 403 — Additional entries specific to a trap network annotation dictionary]
    Annot,
    /// REFERENCE: [Table 169 — Entries in a border effect dictionary]
    Border,
    /// REFERENCE: [Table 194 — Entries in a fixed print dictionary]
    FixedPrint,
    /// REFERENCE:
    /// - [Table 196 — Entries common to all action dictionaries]
    /// - [Table 198 — Entries in a page object’s additional-actions dictionary]
    /// - [Table 200 — Entries in the document catalog’s additional-actions dictionary]
    /// - [Table 202 — Additional entries specific to a go-to action]
    /// - [Table 203 — Additional entries specific to a remote go-to action dictionary]
    /// - [Table 204 — Additional entries specific to an embedded go-to action]
    /// - [Table 205 — Entries specific to a target dictionary]
    /// - [Table 206 — Entries in a GoToDp dictionary]
    /// - [Table 207 — Additional entries specific to a launch action dictionary]
    /// - [Table 208 — Entries in a Microsoft WindowsTM launch parameter dictionary]
    /// - [Table 209 — Additional entries specific to a thread action dictionary]
    /// - [Table 210 — Additional entries specific to a URI action dictionary]
    /// - [Table 212 — Additional entries specific to a sound action dictionary]
    /// - [Table 213 — Additional entries specific to a movie action]
    /// - [Table 214 — Additional entries specific to a hide action]
    /// - [Table 216 — Additional entries specific to named actions]
    /// - [Table 217 — Additional entries specific to a set-OCG-state action dictionary]
    /// - [Table 218 — Additional entries specific to a rendition action]
    /// - [Table 219 — Additional entries specific to a transition action]
    /// - [Table 220 — Additional entries specific to a go-to-3D-view action dictionary]
    /// - [Table 221 — Additional entries specific to an ECMAScript action]
    /// - [Table 222 — Additional entries specific to a rich-media-execute action dictionary]
    /// - [Table 239 — Additional entries specific to a submit-form action dictionary]
    /// - [Table 241 — Additional entries specific to a reset-form action dictionary]
    /// - [Table 243 — Additional entries specific to an import-data action]
    Action,
    /// REFERENCE: [Table 223 — Entries in a RichMediaCommand dictionary]
    RichMediaCommand, // PDF 2.0
    /// REFERENCE: [Table 238 — Entries in a certificate seed value dictionary]
    SVCert,
    /// REFERENCE: [Table 255 — Entries in a signature dictionary]
    Sig,
    /// REFERENCE: [Table 255 — Entries in a signature dictionary]
    DocTimeStamp,
    /// REFERENCE: [Table 256 — Entries in a signature reference dictionary]
    SigRef,
    /// REFERENCE:
    /// - [Table 257 — Entries in the DocMDP transform parameters dictionary]
    /// - [Table 258 — Entries in the UR transform parameters dictionary]
    /// - [Table 259 — Entries in the FieldMDP transform parameters dictionary]
    TransformParams,
    /// REFERENCE: [Table 261 — Entries in the document security store (DSS) dictionary]
    DSS,
    /// REFERENCE: [Table 265 — Entries in a viewport dictionary]
    Viewport,
    /// REFERENCE:
    /// - [Table 266 — Entries in a measure dictionary]
    /// - [Table 267 — Additional entries in a rectilinear measure dictionary]
    /// - [Table 269 — Additional entries in a geospatial measure dictionary]
    Measure,
    /// REFERENCE: [Table 268 — Entries in a number format dictionary]
    NumberFormat,
    /// REFERENCE: [Table 270 — Entries in a geographic coordinate system dictionary]
    GEOGCS, // PDF 2.0
    /// REFERENCE: [Table 271 — Entries in a projected coordinate system dictionary]
    PROJCS, // PDF 2.0
    /// REFERENCE: [Table 272 — Entries in a point data dictionary]
    PtData, // PDF 2.0
    /// REFERENCE: [Table 273 — Entries common to all requirement dictionaries]
    Requirement,
    /// REFERENCE: [Table 276 — Entries in a requirement handler dictionary]
    ReqHandler,
    /// REFERENCE: [Table 277 — Entries common to all rendition dictionaries]
    Rendition,
    /// REFERENCE: [Table 279 — Entries in a media criteria dictionary]
    MediaCriteria,
    /// REFERENCE: [Table 280 — Entries in a minimum bit depth dictionary]
    MinBitDepth,
    /// REFERENCE: [Table 281 — Entries in a minimum screen size dictionary]
    MinScreenSize,
    /// REFERENCE: [Table 284 — Entries common to all media clip dictionaries]
    MediaClip,
    /// REFERENCE: [Table 290 — Entries in a media play parameters dictionary]
    MediaPlayParams,
    /// REFERENCE: [Table 292 — Entries in a media duration dictionary]
    MediaDuration,
    /// REFERENCE: [Table 293 — Entries in a media screen parameters dictionary]
    MediaScreenParams,
    /// REFERENCE: [Table 295 — Entries in a floating window parameters dictionary]
    FWParams,
    /// REFERENCE: [Table 296 — Entries common to all media offset dictionaries]
    MediaOffset,
    /// REFERENCE: [Table 300 — Entries in a timespan dictionary]
    Timespan,
    /// REFERENCE: [Table 301 — Entries in a media players dictionary]
    MediaPlayers,
    /// REFERENCE: [Table 302 — Entries in a media player info dictionary]
    MediaPlayerInfo,
    /// Table 303 — Entries in a software identifier dictionary
    SoftwareIdentifier,
    /// REFERENCE: [Table 308 — Entries in a slideshow dictionary]
    SlideShow, // PDF 1.4
    // TODO Rename to 3D
    /// REFERENCE:
    /// - [Table 309 — Additional entries specific to a 3D annotation]
    /// - [Table 311 — Entries in a 3D stream dictionary]
    _3D, // 3D
    /// REFERENCE: [Table 312 — Entries in an 3D animation style dictionary ]
    _3DAnimationStyle, // 3DAnimationStyle
    /// REFERENCE: [Table 315 — Entries in a 3D view dictionary]
    _3DView, // 3DView
    /// REFERENCE: [Table 317 — Entries in a 3D background dictionary]
    _3DBG, // 3DBG
    /// REFERENCE: [Table 318 — Entries in a render mode dictionary]
    _3DRenderMode, // 3DRenderMode
    /// REFERENCE: [Table 324 — Entries in an external data dictionary used to markup 3D annotations]
    ExData,
    /// REFERENCE: [Table 335 — Entries in a RichMediaActivation dictionary]
    RichMediaActivation, // PDF 2.0
    /// REFERENCE: [Table 336 — Entries in a RichMediaDeactivation dictionary]
    RichMediaDeactivation, // PDF 2.0
    /// REFERENCE: [Table 337 — Entries in a RichMediaAnimation dictionary]
    RichMediaAnimation, // PDF 2.0
    /// REFERENCE: [Table 338 — Entries in a RichMediaPresentation dictionary]
    RichMediaPresentation, // PDF 2.0
    /// REFERENCE: [Table 339 — Entries in a RichMediaWindow dictionary]
    RichMediaWindow, // PDF 2.0
    /// REFERENCE: [Table 340 — Entries in a RichMediaPosition dictionary]
    RichMediaPosition, // PDF 2.0
    /// REFERENCE: [Table 342 — Entries in a RichMediaConfiguration dictionary]
    RichMediaConfiguration, // PDF 2.0
    /// REFERENCE: [Table 343 — Entries in a RichMediaInstance dictionary]
    RichMediaInstance, // PDF 2.0
    /// REFERENCE: [Table 354 — Entries in the structure tree root]
    StructTreeRoot,
    /// REFERENCE: [Table 355 — Entries in a structure element dictionary]
    StructElem,
    /// REFERENCE: [Table 356 — Entries in a namespace dictionary]
    Namespace, // PDF 2.0
    /// REFERENCE: [Table 357 — Entries in a marked-content reference dictionary]
    MCR,
    /// REFERENCE: [Table 358 — Entries in an object reference dictionary]
    OBJR,
    /// REFERENCE: [Table 363 — Property list entries for artifacts]
    Layout,
    /// REFERENCE: [Table 363 — Property list entries for artifacts]
    Background,
    /// REFERENCE:
    /// - [Table 363 — Property list entries for artifacts]
    /// - [Table 385 — Standard artifact attributes]
    Pagination,
    /// REFERENCE: [Table 385 — Standard artifact attributes]
    Inline, // PDF 2.0
    /// REFERENCE:
    /// - [Table 388 — Entries common to all Web Capture content sets]
    /// - [Table 390 — Additional entries specific to a Web Capture image set]
    SpiderContentSet,
    /// REFERENCE: [Table 401 — Entries in an output intent dictionary]
    OutputIntent,
    /// REFERENCE: [Table 408 — Entries in a DPartRoot dictionary]
    DPartRoot, // PDF 2.0
    /// REFERENCE: [Table 409 — Entries in a DPart dictionary]
    DPart, // PDF 2.0
    /// REFERENCE: todo!()
    CryptFilterDecodeParms,
    /// REFERENCE: todo!()
    _3DCrossSection,
    /// REFERENCE: todo!()
    CollectionItem,
    /// REFERENCE: todo!()
    Metadata,
    /// REFERENCE: todo!()
    OPI, // PDF 1.2
    /// REFERENCE: [8.4.5 Graphics state parameter dictionaries]
    ExtGState,
    // ProcSet,
    // Process,
    // Properties,
    // Shading,
    // TODO Found in examples but not in the standard
    // JobTicketContents,
    // TODO(QUESTION): Are both FileSpec and Filespec valid?
    // TODO Found in examples but not in the standard
    // FileSpec,
}
