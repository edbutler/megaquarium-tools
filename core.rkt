#lang racket

(provide
 (all-from-out
  "core/animal.rkt"
  "core/aquarium.rkt"
  "core/tank.rkt"
  "core/search.rkt"
  "core/localization.rkt")
 (all-defined-out))

(require
  "core/animal.rkt"
  "core/aquarium.rkt"
  "core/tank.rkt"
  "core/search.rkt"
  "core/localization.rkt")

; drop the excess stuff from the names, specifically the leading number
; (or/c symbol? string?) -> string?
(define (tweak-id/string id)
  (when (symbol? id) (set! id (symbol->string id)))
  (regexp-replace* #rx"[0-9]+_(.*)" id "\\1"))

; symbol? -> symbol?
(define tweak-id/symbol (compose1 string->symbol tweak-id/string))

(define (game-object-type-id typ)
  (cond
   [(tnktyp? typ) (tnktyp-id typ)]
   [(species? typ) (species-id typ)]
   [else (error "unknown game object type")]))