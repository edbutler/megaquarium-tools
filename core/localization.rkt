#lang racket

(provide
  localization?
  make-localization
  localize
  localize-species
  localize-animal)

(require
  racket/hash
  "animal.rkt")

(struct localization
  ; (hashof string? string?)
  (hash)
  #:transparent)

(define (make-localization loaded-json) (localization (apply hash-union loaded-json)))

(define (localize l10n id) (hash-ref (localization-hash l10n) id))

(define (localize-species l10n spc) (localize l10n (species-id spc)))

(define (localize-animal l10n aml) (localize-species l10n (animal-species aml)))
