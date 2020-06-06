#lang rosette/safe

(provide (all-defined-out))

(struct condition () #:transparent)
(struct species-condition condition (id quantity) #:transparent)
(struct restriction-condition condition (name quantity) #:transparent)
(struct type-condition condition (type quantity) #:transparent)

(struct objective (condition) #:transparent)

(struct market
  ; (listof symbol?)
  (available
  ; (listof symbol?)
   unlockable
  ; (listof (pairof symbol? positive-integer?))
   acquirable)
  #:transparent)

(struct aquarium
  ; (pairof tank? (listof animal?))
 (tanks
  ; market?
  market
  ; (listof objective?)
  objectives)
 #:transparent)

(struct species-spec
   ; species?
  (species
   ; integer?
   count)
  #:transparent)

(struct tank-spec
  (name
   ; (listof species-spec?)
   contents)
  #:transparent)