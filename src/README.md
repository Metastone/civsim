# Reflexion

System: agit sur les entities qui ont les composants requis

Systems en parallele ou par iteration ?
 - en parallele ce sera plus fun et instructif. Chacun dans son thread.
 - thread safety a gérer sur les composants. (perte de perfo si trop d'accès concurrents ?)
 - gestion du partage de resource entre les threads requise ? ou pas vraiment besoin car bien géré par l'OS du moment qu'ils sleep régulièrement ?

System -> map des entities à surveiller (pour ne pas avoir à les chercher en permanence)
==> Il est necessaire de mettre à jour ces maps pour tous les systèmes quand:
    - Creation / suppression d'entité
    - Ajout / suppression de composant dans une entité
==> OK si peu de ces modifs. Sinon, il faudra faire un truc plus chiadé.

